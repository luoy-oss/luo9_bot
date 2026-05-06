// src/plugin/loader.rs
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use libloading::Library;
use tracing::{info, error, warn};

use super::handle::PluginHandle;
use super::manager::PluginInfo;
use super::bus::{Bus, TOPIC_MESSAGE, TOPIC_NOTICE, TOPIC_META_EVENT, TOPIC_TASK, TOPIC_SEND};

/// 插件加载器
pub struct PluginLoader {
    plugin_dir: PathBuf,
}

impl PluginLoader {
    pub fn new<P: AsRef<Path>>(plugin_dir: P) -> Self {
        Self {
            plugin_dir: plugin_dir.as_ref().to_path_buf(),
        }
    }

    /// 加载所有插件，返回 (插件信息列表, 插件句柄列表)
    pub fn load_all(&self) -> Result<(Vec<PluginInfo>, Vec<PluginHandle>), String> {
        let mut infos = Vec::new();
        let mut handles = Vec::new();

        if !self.plugin_dir.exists() {
            fs::create_dir_all(&self.plugin_dir)
                .map_err(|e| format!("创建插件目录失败: {}", e))?;
            info!("已创建插件目录: {:?}", self.plugin_dir);
            return Ok((infos, handles));
        }

        let entries = fs::read_dir(&self.plugin_dir)
            .map_err(|e| format!("读取插件目录失败: {}", e))?;

        for (idx, entry) in entries.enumerate() {
            let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
            let path = entry.path();

            if !Self::is_plugin_file(&path) {
                continue;
            }

            match self.load_single(&path, idx) {
                Ok((info, handle)) => {
                    info!("成功加载插件: {} (ID: {})", info.name, info.id);
                    infos.push(info);
                    if let Some(h) = handle {
                        handles.push(h);
                    }
                }
                Err(e) => {
                    error!("加载插件失败 {:?}: {}", path, e);
                }
            }
        }

        Ok((infos, handles))
    }

    /// 加载单个插件并启动其 plugin_main 线程
    ///
    /// 返回 (PluginInfo, Option<PluginHandle>)。若插件未导出 plugin_main，handle 为 None。
    fn load_single(&self, path: &Path, default_id: usize) -> Result<(PluginInfo, Option<PluginHandle>), String> {
        unsafe {
            let lib = Arc::new(
                Library::new(path)
                    .map_err(|e| format!("加载动态库失败: {}", e))?
            );

            // 检查是否导出了 plugin_main
            let has_main = lib.get::<unsafe extern "C" fn()>(b"plugin_main\0").is_ok();

            let plugin_name = path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            let info = PluginInfo {
                id: default_id,
                name: plugin_name.clone(),
                version: String::new(),
                enabled: has_main,
                path: Some(path.to_string_lossy().to_string()),
                priority: 0,
                block_enabled: false,
                active: has_main,
            };

            let handle = if has_main {
                // 为插件创建 per-topic subscriber
                let subscriber_ids = Self::create_subscribers(&plugin_name);

                let lib_clone = Arc::clone(&lib);
                let subs = subscriber_ids.clone();
                let name = plugin_name.clone();
                let thread_handle = std::thread::spawn(move || {
                    Self::run_plugin(lib_clone, &name, subs);
                });

                Some(PluginHandle {
                    name: plugin_name,
                    lib,
                    thread_handle: Some(thread_handle),
                    subscriber_ids,
                    priority: 0,
                    block_enabled: false,
                    active: true,
                    path: path.to_path_buf(),
                })
            } else {
                warn!("插件 {} 未导出 plugin_main，跳过", plugin_name);
                None
            };

            Ok((info, handle))
        }
    }

    /// 为插件在各 topic 上创建 subscriber
    pub(crate) fn create_subscribers(plugin_name: &str) -> HashMap<String, usize> {
        info!("[loader] 为插件 {} 创建 subscriber...", plugin_name);
        let topics = [TOPIC_MESSAGE, TOPIC_NOTICE, TOPIC_META_EVENT, TOPIC_TASK, TOPIC_SEND];
        let mut ids = HashMap::new();
        for topic in &topics {
            info!("[loader] 插件 {} 尝试订阅 topic: {}", plugin_name, topic);
            match Bus::topic(topic).subscribe() {
                Ok(id) => {
                    ids.insert(topic.to_string(), id);
                    info!("[loader] 插件 {} 订阅成功: {} -> id={}", plugin_name, topic, id);
                }
                Err(e) => {
                    error!("[loader] 插件 {} 订阅失败: {} -> {:?}", plugin_name, topic, e);
                }
            }
        }
        info!("[loader] 插件 {} 最终 subscriber_ids: {:?}", plugin_name, ids);
        ids
    }

    /// 在独立线程中驱动插件的 plugin_main 循环
    ///
    /// 1. 尝试获取 `luo9_init_subscribers` 符号并调用（传递预创建的 subscriber ID）
    /// 2. 获取 `plugin_main` 符号并调用
    /// 3. 插件正常退出或 panic 均只记录日志，不影响主程序
    pub(crate) fn run_plugin(lib: Arc<Library>, plugin_name: &str, subscriber_ids: HashMap<String, usize>) {
        unsafe {
            // 尝试调用 luo9_init_subscribers 传递预创建的 subscriber ID
            // 使用 repr(C) 兼容的裸结构体定义，避免跨 crate 类型匹配问题
            #[repr(C)]
            struct PluginSubscribersRaw {
                message_sub_id: i32,
                meta_event_sub_id: i32,
                notice_sub_id: i32,
                task_sub_id: i32,
                send_sub_id: i32,
            }

            type InitSubscribersFn = unsafe extern "C" fn(*const PluginSubscribersRaw);
            let init_result = lib.get::<InitSubscribersFn>(b"luo9_init_subscribers\0");
            match init_result {
                Ok(init_fn) => {
                    let subs = PluginSubscribersRaw {
                        message_sub_id: subscriber_ids.get(TOPIC_MESSAGE).copied().unwrap_or(0) as i32,
                        meta_event_sub_id: subscriber_ids.get(TOPIC_META_EVENT).copied().unwrap_or(0) as i32,
                        notice_sub_id: subscriber_ids.get(TOPIC_NOTICE).copied().unwrap_or(0) as i32,
                        task_sub_id: subscriber_ids.get(TOPIC_TASK).copied().unwrap_or(0) as i32,
                        send_sub_id: subscriber_ids.get(TOPIC_SEND).copied().unwrap_or(0) as i32,
                    };
                    info!("[loader] 插件 {} 传递 subscriber 映射: msg={}, notice={}, meta={}, task={}, send={}",
                        plugin_name, subs.message_sub_id, subs.notice_sub_id, subs.meta_event_sub_id, subs.task_sub_id, subs.send_sub_id);
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

    fn is_plugin_file(path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }

        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        #[cfg(target_os = "linux")]
        return ext == "so";

        #[cfg(target_os = "windows")]
        return ext == "dll";

        #[allow(unreachable_code)]
        false
    }

    /// 重新加载单个插件（卸载后重新加载）
    pub fn reload_single(&self, path: &Path, id: usize) -> Result<(PluginInfo, Option<PluginHandle>), String> {
        self.load_single(path, id)
    }
}

/// 独立的单个插件加载函数（供 enable_plugin / reload_plugin 调用）
///
/// 与 `PluginLoader::load_single` 逻辑相同，但不依赖 PluginLoader 实例。
pub fn load_single_plugin(path: &Path, default_id: usize) -> Result<(PluginInfo, Option<PluginHandle>), String> {
    unsafe {
        let lib = Arc::new(
            Library::new(path)
                .map_err(|e| format!("加载动态库失败: {}", e))?
        );

        let has_main = lib.get::<unsafe extern "C" fn()>(b"plugin_main\0").is_ok();

        let plugin_name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let info = PluginInfo {
            id: default_id,
            name: plugin_name.clone(),
            version: String::new(),
            enabled: has_main,
            path: Some(path.to_string_lossy().to_string()),
            priority: 0,
            block_enabled: false,
            active: has_main,
        };

        let handle = if has_main {
            let subscriber_ids = PluginLoader::create_subscribers(&plugin_name);

            let lib_clone = Arc::clone(&lib);
            let subs = subscriber_ids.clone();
            let name = plugin_name.clone();
            let thread_handle = std::thread::spawn(move || {
                PluginLoader::run_plugin(lib_clone, &name, subs);
            });

            Some(PluginHandle {
                name: plugin_name,
                lib,
                thread_handle: Some(thread_handle),
                subscriber_ids,
                priority: 0,
                block_enabled: false,
                active: true,
                path: path.to_path_buf(),
            })
        } else {
            warn!("插件 {} 未导出 plugin_main，跳过", plugin_name);
            None
        };

        Ok((info, handle))
    }
}
