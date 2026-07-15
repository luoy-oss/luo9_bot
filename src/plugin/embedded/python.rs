// src/plugin/embedded/python.rs
//! Python 嵌入式运行时（PyO3）
//!
//! 通过 PyO3 嵌入 CPython 解释器，在宿主进程内执行 Python 插件。
//! Python SDK 通过 ctypes 直接调用 luo9_core.dll FFI。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;
use tracing::{info, error};

use crate::plugin::runtime::PluginRuntime;

/// 全局一次性初始化 Python 解释器（free-threaded 模式）
static PYTHON_INIT: std::sync::Once = std::sync::Once::new();

/// Python 插件运行时
pub struct PythonRuntime {
    plugin_path: PathBuf,
    plugin_name: String,
    thread_handle: Option<JoinHandle<()>>,
    subscriber_ids: HashMap<String, usize>,
}

impl PythonRuntime {
    pub fn new(path: &Path) -> Result<Self, String> {
        let file_name = path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        let name = file_name.trim_end_matches(".py").to_string();

        Ok(Self {
            plugin_path: path.to_path_buf(),
            plugin_name: name,
            thread_handle: None,
            subscriber_ids: HashMap::new(),
        })
    }
}

impl PluginRuntime for PythonRuntime {
    fn start(&mut self, subscriber_ids: HashMap<String, usize>) -> Result<(), String> {
        self.subscriber_ids = subscriber_ids;

        let plugin_path = self.plugin_path.clone();
        let plugin_name = self.plugin_name.clone();
        let subs = self.subscriber_ids.clone();

        self.thread_handle = Some(std::thread::spawn(move || {
            run_python_plugin(&plugin_path, &plugin_name, &subs);
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

/// 在独立线程中执行 Python 插件
fn run_python_plugin(
    plugin_path: &Path,
    plugin_name: &str,
    subscriber_ids: &HashMap<String, usize>,
) {
    use pyo3::prelude::*;
    use pyo3::types::PyDict;

    // 初始化 Python 解释器（仅首次调用生效）
    // Python SDK 通过 GetModuleHandle/RTLD_NOLOAD 自动复用宿主已加载的 luo9_core
    PYTHON_INIT.call_once(|| {
        pyo3::prepare_freethreaded_python();
        // 重置 SIGINT 为系统默认行为（终止进程），防止 Python 的 KeyboardInterrupt
        // 干扰宿主的 tokio 信号处理。prepare_freethreaded_python 会安装 Python 的
        // SIGINT handler，必须在初始化后立即覆盖。
        reset_sigint_handler();
    });

    let plugin_name_owned = plugin_name.to_string();
    let plugin_path_owned = plugin_path.to_path_buf();
    let subs = subscriber_ids.clone();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        Python::with_gil(|py| -> PyResult<()> {
            // 将 luo9_sdk Python 包目录添加到 sys.path
            let sys = py.import("sys")?;
            let path = sys.getattr("path")?;

            // 插件所在目录（插件可能 import 同目录模块）
            if let Some(parent) = plugin_path_owned.parent() {
                let parent_str = parent.to_string_lossy().to_string();
                path.call_method1("insert", (0, parent_str))?;
            }

            // luo9_sdk 目录（默认位于插件目录的上级 sdk/python/）
            // 也搜索工作目录下的常见位置
            let cwd = std::env::current_dir().unwrap_or_default();
            let sdk_candidates = [
                cwd.join("sdk").join("python"),
                cwd.join("..").join("sdk").join("python"),
            ];
            for sdk_dir in &sdk_candidates {
                if sdk_dir.exists() {
                    let sdk_str = sdk_dir.to_string_lossy().to_string();
                    info!("[python] 已添加 SDK 路径: {}", sdk_str);
                    path.call_method1("insert", (0, sdk_str))?;
                    break;
                }
            }

            // 注入预分配的 subscriber ID 到 luo9_sdk.bus 模块
            if !subs.is_empty() {
                let luo9_bus = py.import("luo9_sdk.bus")?;
                let py_subs = PyDict::new(py);
                for (topic, &id) in &subs {
                    py_subs.set_item(topic, id)?;
                }
                luo9_bus.call_method1("init_subscribers", (py_subs,))?;
                info!("[python] 已注入 subscriber 映射: {:?}", subs);
            }

            // 加载插件模块
            let stem = plugin_path_owned.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("plugin");
            let plugin_mod = py.import(stem)?;

            // 调用 plugin_main()（包装错误捕获 + 强制刷新输出）
            let main_fn = plugin_mod.getattr("plugin_main")?;
            info!("[python] 插件 {} 开始执行", plugin_name_owned);

            // 注入调试代码：强制 unbuffered 输出
            let sys = py.import("sys")?;
            let stderr = sys.getattr("stderr")?;
            let _ = stderr.call_method1("write", (format!("[python-debug] 插件 {} 启动, subs={:?}\n", plugin_name_owned, subs),));
            let _ = stderr.call_method0("flush");

            // 用 try/except 包装 plugin_main，捕获异常并强制输出
            let traceback_mod = py.import("traceback")?;

            match main_fn.call0() {
                Ok(_) => {
                    let _ = stderr.call_method1("write", (format!("[python-debug] 插件 {} 正常退出\n", plugin_name_owned),));
                    let _ = stderr.call_method0("flush");
                    info!("[python] 插件 {} 已正常退出", plugin_name_owned);
                }
                Err(e) => {
                    let _ = stderr.call_method1("write", (format!("[python-debug] 插件 {} 异常: {}\n", plugin_name_owned, e),));
                    let _ = traceback_mod.call_method0("print_exc");
                    let _ = stderr.call_method0("flush");
                    return Err(e);
                }
            }
            Ok(())
        })
    }));

    match result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => error!("[python] 插件 {} Python 错误: {}", plugin_name, e),
        Err(e) => error!("[python] 插件 {} panic: {:?}", plugin_name, e),
    }
}

/// 重置 SIGINT 为系统默认行为（终止进程）
///
/// Python 初始化时会安装自己的 SIGINT handler，将 Ctrl+C 转换为
/// KeyboardInterrupt 异常。这会干扰宿主的 tokio 信号处理，导致进程无法正常退出。
/// 此函数在 Python 初始化后立即调用，将 SIGINT 恢复为默认行为。
fn reset_sigint_handler() {
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGINT, libc::SIG_DFL);
    }
    #[cfg(windows)]
    {
        // Windows: 添加一个返回 FALSE 的 Ctrl+C handler，让信号传递到默认处理器（终止进程）
        // Python 的 handler 返回 TRUE 会吞掉信号，我们追加一个 FALSE handler 来恢复默认行为
        unsafe extern "system" {
            fn SetConsoleCtrlHandler(
                handler: Option<unsafe extern "system" fn(u32) -> i32>,
                add: i32,
            ) -> i32;
        }
        unsafe extern "system" fn ctrl_handler(_ctrl_type: u32) -> i32 {
            0 // FALSE = 不处理，让系统执行默认行为（终止进程）
        }
        unsafe { SetConsoleCtrlHandler(Some(ctrl_handler), 1) };
    }
}
