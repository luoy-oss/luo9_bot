// src/plugin/embedded/quickjs.rs
//! QuickJS 嵌入式运行时
//!
//! 内嵌 QuickJS 引擎（~210KB，零外部依赖），执行 JavaScript 插件。
//! 通过注册 `__luo9_core` 全局对象暴露所有 luo9_core FFI 函数，
//! SDK 的 `core.js` 读取此对象完成绑定。

use std::collections::HashMap;
use std::ffi::CStr;
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;
use tracing::{error, info};

use crate::plugin::runtime::PluginRuntime;

/// QuickJS 插件运行时
pub struct QuickjsRuntime {
    plugin_path: PathBuf,
    plugin_name: String,
    thread_handle: Option<JoinHandle<()>>,
    subscriber_ids: HashMap<String, usize>,
}

impl QuickjsRuntime {
    pub fn new(path: &Path) -> Result<Self, String> {
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        let name = file_name.trim_end_matches(".js").to_string();

        Ok(Self {
            plugin_path: path.to_path_buf(),
            plugin_name: name,
            thread_handle: None,
            subscriber_ids: HashMap::new(),
        })
    }
}

impl PluginRuntime for QuickjsRuntime {
    fn start(&mut self, subscriber_ids: HashMap<String, usize>) -> Result<(), String> {
        self.subscriber_ids = subscriber_ids;

        let plugin_path = self.plugin_path.clone();
        let plugin_name = self.plugin_name.clone();
        let subs = self.subscriber_ids.clone();

        self.thread_handle = Some(std::thread::spawn(move || {
            run_quickjs_plugin(&plugin_path, &plugin_name, &subs);
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
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
        Ok(())
    }

    fn force_stop(&mut self) -> Result<(), String> {
        self.stop()
    }

    fn subscriber_ids(&self) -> &HashMap<String, usize> {
        &self.subscriber_ids
    }
}

/// 从 C 指针安全读取字符串，null 返回 None
unsafe fn c_ptr_to_opt_string(ptr: *const std::ffi::c_char) -> Option<String> {
    if ptr.is_null() {
        None
    } else {
        unsafe { Some(CStr::from_ptr(ptr).to_string_lossy().into_owned()) }
    }
}

/// 在独立线程中执行 QuickJS 插件
fn run_quickjs_plugin(
    plugin_path: &Path,
    plugin_name: &str,
    subscriber_ids: &HashMap<String, usize>,
) {
    use rquickjs::{Context, Runtime};

    let runtime = match Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            error!("[quickjs] 创建运行时失败: {}", e);
            return;
        }
    };

    let ctx = match Context::full(&runtime) {
        Ok(c) => c,
        Err(e) => {
            error!("[quickjs] 创建上下文失败: {}", e);
            return;
        }
    };

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        ctx.with(|ctx| -> Result<(), String> {
            let globals = ctx.globals();

            // ── 注册 __luo9_core 全局对象 ─────────────────────────────

            let core = rquickjs::Object::new(ctx.clone())
                .map_err(|e| format!("创建 __luo9_core 对象失败: {}", e))?;

            // luo9_bus_init
            core.set(
                "luo9_bus_init",
                rquickjs::Function::new(ctx.clone(), || unsafe { luo9_core::luo9_bus_init() })
                    .map_err(|e| format!("注册 luo9_bus_init 失败: {}", e))?,
            )
            .map_err(|e| format!("设置 luo9_bus_init 失败: {}", e))?;

            // luo9_bus_subscribe(topic)
            core.set(
                "luo9_bus_subscribe",
                rquickjs::Function::new(ctx.clone(), |topic: String| {
                    let c_topic = std::ffi::CString::new(topic).unwrap();
                    unsafe { luo9_core::luo9_bus_subscribe(c_topic.as_ptr()) }
                })
                .map_err(|e| format!("注册 luo9_bus_subscribe 失败: {}", e))?,
            )
            .map_err(|e| format!("设置 luo9_bus_subscribe 失败: {}", e))?;

            // luo9_bus_unsubscribe(topic, sub_id)
            core.set(
                "luo9_bus_unsubscribe",
                rquickjs::Function::new(ctx.clone(), |topic: String, sub_id: i32| {
                    let c_topic = std::ffi::CString::new(topic).unwrap();
                    unsafe { luo9_core::luo9_bus_unsubscribe(c_topic.as_ptr(), sub_id) }
                })
                .map_err(|e| format!("注册 luo9_bus_unsubscribe 失败: {}", e))?,
            )
            .map_err(|e| format!("设置 luo9_bus_unsubscribe 失败: {}", e))?;

            // luo9_bus_publish(topic, payload)
            core.set(
                "luo9_bus_publish",
                rquickjs::Function::new(ctx.clone(), |topic: String, payload: String| {
                    let c_topic = std::ffi::CString::new(topic).unwrap();
                    let c_payload = std::ffi::CString::new(payload).unwrap();
                    unsafe { luo9_core::luo9_bus_publish(c_topic.as_ptr(), c_payload.as_ptr()) }
                })
                .map_err(|e| format!("注册 luo9_bus_publish 失败: {}", e))?,
            )
            .map_err(|e| format!("设置 luo9_bus_publish 失败: {}", e))?;

            // luo9_bus_pop(topic, sub_id) -> string|null
            core.set(
                "luo9_bus_pop",
                rquickjs::Function::new(ctx.clone(), move |topic: String, sub_id: i32| {
                    let c_topic = std::ffi::CString::new(topic).unwrap();
                    let ptr = unsafe { luo9_core::luo9_bus_pop(c_topic.as_ptr(), sub_id) };
                    match unsafe { c_ptr_to_opt_string(ptr) } {
                        Some(s) => {
                            unsafe { luo9_core::luo9_bus_free_string(ptr) };
                            Some(s)
                        }
                        None => None,
                    }
                })
                .map_err(|e| format!("注册 luo9_bus_pop 失败: {}", e))?,
            )
            .map_err(|e| format!("设置 luo9_bus_pop 失败: {}", e))?;

            // luo9_bus_wait_pop(topic, sub_id) -> string|null
            core.set(
                "luo9_bus_wait_pop",
                rquickjs::Function::new(ctx.clone(), move |topic: String, sub_id: i32| {
                    let c_topic = std::ffi::CString::new(topic).unwrap();
                    let ptr = unsafe { luo9_core::luo9_bus_wait_pop(c_topic.as_ptr(), sub_id) };
                    match unsafe { c_ptr_to_opt_string(ptr) } {
                        Some(s) => {
                            unsafe { luo9_core::luo9_bus_free_string(ptr) };
                            Some(s)
                        }
                        None => None,
                    }
                })
                .map_err(|e| format!("注册 luo9_bus_wait_pop 失败: {}", e))?,
            )
            .map_err(|e| format!("设置 luo9_bus_wait_pop 失败: {}", e))?;

            // luo9_bus_free_string(ptr)
            core.set(
                "luo9_bus_free_string",
                rquickjs::Function::new(ctx.clone(), |ptr: u64| unsafe {
                    luo9_core::luo9_bus_free_string(ptr as *mut std::ffi::c_char)
                })
                .map_err(|e| format!("注册 luo9_bus_free_string 失败: {}", e))?,
            )
            .map_err(|e| format!("设置 luo9_bus_free_string 失败: {}", e))?;

            // luo9_command_create(msg, cmd_name, mode, prefix) -> handle
            core.set(
                "luo9_command_create",
                rquickjs::Function::new(
                    ctx.clone(),
                    |msg: String, cmd_name: String, mode: i32, prefix: String| {
                        let c_msg = std::ffi::CString::new(msg).unwrap();
                        let c_cmd = std::ffi::CString::new(cmd_name).unwrap();
                        let prefix_byte = prefix.as_bytes().first().copied().unwrap_or(b'/');
                        let handle = unsafe {
                            luo9_core::luo9_command_create(
                                c_msg.as_ptr(),
                                c_cmd.as_ptr(),
                                mode,
                                prefix_byte as std::ffi::c_char,
                            )
                        };
                        handle as u64
                    },
                )
                .map_err(|e| format!("注册 luo9_command_create 失败: {}", e))?,
            )
            .map_err(|e| format!("设置 luo9_command_create 失败: {}", e))?;

            // luo9_command_free(handle)
            core.set(
                "luo9_command_free",
                rquickjs::Function::new(ctx.clone(), |handle: u64| unsafe {
                    luo9_core::luo9_command_free(handle as *mut std::ffi::c_void)
                })
                .map_err(|e| format!("注册 luo9_command_free 失败: {}", e))?,
            )
            .map_err(|e| format!("设置 luo9_command_free 失败: {}", e))?;

            // luo9_command_get_name(handle) -> string|null
            core.set(
                "luo9_command_get_name",
                rquickjs::Function::new(ctx.clone(), |handle: u64| {
                    let ptr = unsafe {
                        luo9_core::luo9_command_get_name(handle as *mut std::ffi::c_void)
                    };
                    match unsafe { c_ptr_to_opt_string(ptr) } {
                        Some(s) => {
                            unsafe { luo9_core::luo9_free_string(ptr) };
                            Some(s)
                        }
                        None => None,
                    }
                })
                .map_err(|e| format!("注册 luo9_command_get_name 失败: {}", e))?,
            )
            .map_err(|e| format!("设置 luo9_command_get_name 失败: {}", e))?;

            // luo9_command_get_args_raw(handle) -> string|null
            core.set(
                "luo9_command_get_args_raw",
                rquickjs::Function::new(ctx.clone(), |handle: u64| {
                    let ptr = unsafe {
                        luo9_core::luo9_command_get_args_raw(handle as *mut std::ffi::c_void)
                    };
                    match unsafe { c_ptr_to_opt_string(ptr) } {
                        Some(s) => {
                            unsafe { luo9_core::luo9_free_string(ptr) };
                            Some(s)
                        }
                        None => None,
                    }
                })
                .map_err(|e| format!("注册 luo9_command_get_args_raw 失败: {}", e))?,
            )
            .map_err(|e| format!("设置 luo9_command_get_args_raw 失败: {}", e))?;

            // luo9_command_has_args(handle) -> int
            core.set(
                "luo9_command_has_args",
                rquickjs::Function::new(ctx.clone(), |handle: u64| unsafe {
                    luo9_core::luo9_command_has_args(handle as *mut std::ffi::c_void)
                })
                .map_err(|e| format!("注册 luo9_command_has_args 失败: {}", e))?,
            )
            .map_err(|e| format!("设置 luo9_command_has_args 失败: {}", e))?;

            // luo9_command_args_count(handle) -> int
            core.set(
                "luo9_command_args_count",
                rquickjs::Function::new(ctx.clone(), |handle: u64| unsafe {
                    luo9_core::luo9_command_args_count(handle as *mut std::ffi::c_void)
                })
                .map_err(|e| format!("注册 luo9_command_args_count 失败: {}", e))?,
            )
            .map_err(|e| format!("设置 luo9_command_args_count 失败: {}", e))?;

            // luo9_command_get_arg(handle, index) -> string|null
            core.set(
                "luo9_command_get_arg",
                rquickjs::Function::new(ctx.clone(), |handle: u64, index: u32| {
                    let ptr = unsafe {
                        luo9_core::luo9_command_get_arg(handle as *mut std::ffi::c_void, index)
                    };
                    match unsafe { c_ptr_to_opt_string(ptr) } {
                        Some(s) => {
                            unsafe { luo9_core::luo9_free_string(ptr) };
                            Some(s)
                        }
                        None => None,
                    }
                })
                .map_err(|e| format!("注册 luo9_command_get_arg 失败: {}", e))?,
            )
            .map_err(|e| format!("设置 luo9_command_get_arg 失败: {}", e))?;

            // luo9_free_string(ptr)
            core.set(
                "luo9_free_string",
                rquickjs::Function::new(ctx.clone(), |ptr: u64| unsafe {
                    luo9_core::luo9_free_string(ptr as *mut std::ffi::c_char)
                })
                .map_err(|e| format!("注册 luo9_free_string 失败: {}", e))?,
            )
            .map_err(|e| format!("设置 luo9_free_string 失败: {}", e))?;

            // luo9_version() -> string
            core.set(
                "luo9_version",
                rquickjs::Function::new(ctx.clone(), || {
                    let ptr = unsafe { luo9_core::luo9_version() };
                    unsafe { c_ptr_to_opt_string(ptr) }.unwrap_or_default()
                })
                .map_err(|e| format!("注册 luo9_version 失败: {}", e))?,
            )
            .map_err(|e| format!("设置 luo9_version 失败: {}", e))?;

            // 将 __luo9_core 注册到全局作用域
            globals
                .set("__luo9_core", core)
                .map_err(|e| format!("设置 __luo9_core 全局对象失败: {}", e))?;

            // ── 注入预分配的 subscriber ID ───────────────────────────

            let subs_obj = rquickjs::Object::new(ctx.clone())
                .map_err(|e| format!("创建 subscribers 对象失败: {}", e))?;
            subscriber_ids.iter().try_for_each(|(topic, &id)| {
                subs_obj
                    .set(topic.as_str(), id as i32)
                    .map_err(|e| format!("设置 subscriber 失败: {}", e))
            })?;
            globals
                .set("__luo9_subscribers", subs_obj)
                .map_err(|e| format!("设置 __luo9_subscribers 失败: {}", e))?;

            info!(
                "[quickjs] 已注册 __luo9_core FFI 绑定，subscriber: {:?}",
                subscriber_ids
            );

            // ── 加载并执行插件脚本 ───────────────────────────────────

            let script = std::fs::read_to_string(plugin_path)
                .map_err(|e| format!("读取插件文件失败: {}", e))?;

            info!("[quickjs] 插件 {} 开始执行", plugin_name);

            ctx.eval::<(), _>(script.as_str())
                .map_err(|e| format!("执行插件脚本失败: {}", e))?;

            // 调用 plugin_main()
            let main_fn: rquickjs::Function = globals
                .get("plugin_main")
                .map_err(|e| format!("获取 plugin_main 失败: {}", e))?;

            main_fn
                .call::<_, ()>(())
                .map_err(|e| format!("调用 plugin_main 失败: {}", e))?;

            info!("[quickjs] 插件 {} 已正常退出", plugin_name);
            Ok(())
        })
    }));

    match result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => error!("[quickjs] 插件 {} 错误: {}", plugin_name, e),
        Err(e) => error!("[quickjs] 插件 {} panic: {:?}", plugin_name, e),
    }
}

// ── luo9_core FFI 声明 ────────────────────────────────────────────

mod luo9_core {
    use std::ffi::c_char;

    unsafe extern "C" {
        pub fn luo9_bus_init() -> i32;
        pub fn luo9_bus_subscribe(topic: *const c_char) -> i32;
        pub fn luo9_bus_unsubscribe(topic: *const c_char, subscriber_id: i32) -> i32;
        pub fn luo9_bus_publish(topic: *const c_char, payload: *const c_char) -> i32;
        pub fn luo9_bus_pop(topic: *const c_char, subscriber_id: i32) -> *mut c_char;
        pub fn luo9_bus_wait_pop(topic: *const c_char, subscriber_id: i32) -> *mut c_char;
        pub fn luo9_bus_free_string(ptr: *mut c_char);

        pub fn luo9_command_create(
            msg: *const c_char,
            cmd_name: *const c_char,
            mode: i32,
            prefix: c_char,
        ) -> *mut std::ffi::c_void;
        pub fn luo9_command_free(handle: *mut std::ffi::c_void);
        pub fn luo9_command_get_name(handle: *mut std::ffi::c_void) -> *mut c_char;
        pub fn luo9_command_get_args_raw(handle: *mut std::ffi::c_void) -> *mut c_char;
        pub fn luo9_command_has_args(handle: *mut std::ffi::c_void) -> i32;
        pub fn luo9_command_args_count(handle: *mut std::ffi::c_void) -> i32;
        pub fn luo9_command_get_arg(handle: *mut std::ffi::c_void, index: u32) -> *mut c_char;
        pub fn luo9_free_string(ptr: *mut c_char);

        pub fn luo9_version() -> *const c_char;
    }
}
