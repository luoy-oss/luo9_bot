// src/plugin/native_runtime.rs
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread::JoinHandle;
use libloading::Library;
use tracing::{info, warn, error};

use super::bus::Bus;
use super::runtime::PluginRuntime;

/// 原生 DLL/SO 插件运行时
///
/// 通过 libloading 加载动态库，在独立线程中调用 plugin_main()。
/// 插件通过 luo9_core.dll FFI 直接与宿主共享 Bus 单例。
pub struct NativeRuntime {
    pub name: String,
    pub lib: Arc<Library>,
    pub thread_handle: Option<JoinHandle<()>>,
    pub subscriber_ids: HashMap<String, usize>,
    pub path: PathBuf,
}

impl PluginRuntime for NativeRuntime {
    fn start(&mut self, subscriber_ids: HashMap<String, usize>) -> Result<(), String> {
        self.subscriber_ids = subscriber_ids;

        let lib = Arc::clone(&self.lib);
        let subs = self.subscriber_ids.clone();
        let name = self.name.clone();

        self.thread_handle = Some(std::thread::spawn(move || {
            Self::run_plugin(lib, &name, subs);
        }));

        Ok(())
    }

    fn is_alive(&self) -> bool {
        match &self.thread_handle {
            Some(handle) => !handle.is_finished(),
            None => false,
        }
    }

    fn stop(&mut self) -> Result<(), String> {
        // 1. 取消所有订阅（触发 sentinel，唤醒阻塞的 wait_pop）
        self.unsubscribe_all();

        // 2. 等待线程退出（500ms 超时）
        let exited = self.wait_exit(std::time::Duration::from_millis(500));
        if !exited {
            warn!("插件 {} 线程在 500ms 内未退出", self.name);
        }
        Ok(())
    }

    fn force_stop(&mut self) -> Result<(), String> {
        self.unsubscribe_all();
        let exited = self.wait_exit(std::time::Duration::from_secs(5));
        if !exited {
            warn!("插件 {} 线程在 5s 内未退出", self.name);
        }
        Ok(())
    }

    fn subscriber_ids(&self) -> &HashMap<String, usize> {
        &self.subscriber_ids
    }
}

impl NativeRuntime {
    /// 创建新的原生运行时（加载 DLL/SO）
    pub unsafe fn new(path: &PathBuf) -> Result<Self, String> {
        let lib = unsafe {
            Arc::new(
                Library::new(path)
                    .map_err(|e| format!("加载动态库失败: {}", e))?
            )
        };

        let file_name = path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        let name = extract_display_name(file_name);

        Ok(Self {
            name,
            lib,
            thread_handle: None,
            subscriber_ids: HashMap::new(),
            path: path.clone(),
        })
    }

    /// 检查是否导出了 plugin_main 符号
    pub unsafe fn has_plugin_main(&self) -> bool {
        unsafe { self.lib.get::<unsafe extern "C" fn()>(b"plugin_main\0").is_ok() }
    }

    /// 取消所有 topic 的订阅
    fn unsubscribe_all(&self) {
        for (topic, &sub_id) in &self.subscriber_ids {
            if let Err(e) = Bus::topic(topic).unsubscribe(sub_id) {
                warn!("插件 {} 取消订阅 {} 失败: {:?}", self.name, topic, e);
            } else {
                info!("插件 {} 已取消订阅 {} (sub_id={})", self.name, topic, sub_id);
            }
        }
    }

    /// 等待插件线程退出
    fn wait_exit(&mut self, timeout: std::time::Duration) -> bool {
        if let Some(handle) = self.thread_handle.take() {
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let _ = handle.join();
                let _ = tx.send(());
            });
            rx.recv_timeout(timeout).is_ok()
        } else {
            true
        }
    }

    /// 在独立线程中驱动插件的 plugin_main
    fn run_plugin(lib: Arc<Library>, plugin_name: &str, subscriber_ids: HashMap<String, usize>) {
        unsafe {
            // 尝试调用 luo9_init_subscribers 传递预创建的 subscriber ID
            #[repr(C)]
            struct PluginSubscribersRaw {
                message_sub_id: i32,
                meta_event_sub_id: i32,
                notice_sub_id: i32,
                request_sub_id: i32,
                task_sub_id: i32,
                send_sub_id: i32,
            }

            type InitSubscribersFn = unsafe extern "C" fn(*const PluginSubscribersRaw);
            let init_result = lib.get::<InitSubscribersFn>(b"luo9_init_subscribers\0");
            match init_result {
                Ok(init_fn) => {
                    use super::bus::{TOPIC_MESSAGE, TOPIC_NOTICE, TOPIC_META_EVENT, TOPIC_REQUEST, TOPIC_TASK, TOPIC_SEND};
                    let subs = PluginSubscribersRaw {
                        message_sub_id: subscriber_ids.get(TOPIC_MESSAGE).copied().unwrap_or(0) as i32,
                        meta_event_sub_id: subscriber_ids.get(TOPIC_META_EVENT).copied().unwrap_or(0) as i32,
                        notice_sub_id: subscriber_ids.get(TOPIC_NOTICE).copied().unwrap_or(0) as i32,
                        request_sub_id: subscriber_ids.get(TOPIC_REQUEST).copied().unwrap_or(0) as i32,
                        task_sub_id: subscriber_ids.get(TOPIC_TASK).copied().unwrap_or(0) as i32,
                        send_sub_id: subscriber_ids.get(TOPIC_SEND).copied().unwrap_or(0) as i32,
                    };
                    info!("[native] 插件 {} 传递 subscriber 映射: msg={}, notice={}, meta={}, request={}, task={}, send={}",
                        plugin_name, subs.message_sub_id, subs.notice_sub_id, subs.meta_event_sub_id,
                        subs.request_sub_id, subs.task_sub_id, subs.send_sub_id);
                    init_fn(&subs);
                    info!("插件 {} 已初始化 subscriber 映射", plugin_name);
                }
                Err(_) => {
                    warn!("插件 {} 未导出 luo9_init_subscribers，将使用默认订阅", plugin_name);
                }
            }

            // 获取 plugin_main 符号
            let plugin_main: libloading::Symbol<unsafe extern "C" fn()> = match lib.get(b"plugin_main\0") {
                Ok(s) => s,
                Err(e) => {
                    error!("插件 {} 获取 plugin_main 失败: {}", plugin_name, e);
                    return;
                }
            };

            info!("插件 {} 线程启动", plugin_name);

            let name = plugin_name.to_string();
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                plugin_main();
            }));

            match result {
                Ok(()) => info!("插件 {} 已正常退出", name),
                Err(e) => error!("插件 {} panic: {:?}", name, e),
            }
        }
    }
}

/// 从文件名提取显示名称（去掉 lib 前缀和 .so/.dll 后缀）
pub fn extract_display_name(file_name: &str) -> String {
    let name = if file_name.ends_with(".so") {
        file_name.trim_end_matches(".so")
    } else if file_name.ends_with(".dll") {
        file_name.trim_end_matches(".dll")
    } else {
        file_name
    };
    let name = name.strip_prefix("lib").unwrap_or(name);
    name.to_string()
}
