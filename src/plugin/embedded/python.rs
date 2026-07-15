// src/plugin/embedded/python.rs
//! Python 嵌入式运行时（PyO3）
//!
//! 通过 PyO3 嵌入 CPython 解释器，在宿主进程内执行 Python 插件。
//! Python SDK 通过 ctypes 直接调用 luo9_core.dll FFI。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;
use tracing::{info, error};

#[cfg(unix)]
use std::sync::atomic::{AtomicUsize, Ordering};
#[cfg(unix)]
use std::sync::OnceLock;

use crate::plugin::runtime::PluginRuntime;

/// 全局一次性初始化 Python 解释器（free-threaded 模式）
static PYTHON_INIT: std::sync::Once = std::sync::Once::new();

// ──────────────────────────────── SIGINT 保护 ────────────────────────────────

/// 进程级 SIGINT handler 的保存槽（仅首次 guard 写入）
#[cfg(unix)]
static SAVED_SIGINT_HANDLER: OnceLock<libc::sigaction> = OnceLock::new();

/// 当前存活的 SigintGuard 数量（首个 guard 保存 handler，最后一个恢复）
#[cfg(unix)]
static SIGINT_GUARD_COUNT: AtomicUsize = AtomicUsize::new(0);

/// RAII 守卫：在首尾配对保存 / 恢复进程级 SIGINT handler。
///
/// Python 代码（`import signal`、第三方库等）会通过 `sigaction()` 覆盖
/// tokio/signal_hook 注册的 handler，导致主进程的 `ctrl_c()` 失效。
/// 本 guard 保证 Python 代码执行期间的 handler 变更是临时的：
///   - 第一个 guard 创建时保存当前 handler（tokio 的）
///   - 最后一个 guard drop 时恢复
/// 多个 Python 插件线程并发时也能正确工作。
#[cfg(unix)]
struct SigintGuard;

#[cfg(unix)]
impl SigintGuard {
    fn protect() -> Self {
        if SIGINT_GUARD_COUNT.fetch_add(1, Ordering::SeqCst) == 0 {
            // 首个 guard：保存当前 handler
            unsafe {
                let mut sa: libc::sigaction = std::mem::zeroed();
                libc::sigaction(libc::SIGINT, std::ptr::null(), &mut sa);
                let _ = SAVED_SIGINT_HANDLER.set(sa);
            }
        }
        Self
    }
}

#[cfg(unix)]
impl Drop for SigintGuard {
    fn drop(&mut self) {
        if SIGINT_GUARD_COUNT.fetch_sub(1, Ordering::SeqCst) == 1 {
            // 最后一个 guard：恢复原始 handler
            if let Some(handler) = SAVED_SIGINT_HANDLER.get() {
                unsafe {
                    libc::sigaction(
                        libc::SIGINT,
                        handler as *const _,
                        std::ptr::null_mut(),
                    );
                }
            }
        }
    }
}

/// 在当前线程屏蔽 SIGINT，使该信号只能投递到主线程（由 tokio 处理）。
#[cfg(unix)]
fn block_sigint_on_current_thread() {
    unsafe {
        let mut mask: libc::sigset_t = std::mem::zeroed();
        libc::sigemptyset(&mut mask);
        libc::sigaddset(&mut mask, libc::SIGINT);
        libc::pthread_sigmask(libc::SIG_BLOCK, &mask, std::ptr::null_mut());
    }
}

// ─────────────────────────── Windows 兼容（no-op）───────────────────────────

#[cfg(not(unix))]
struct SigintGuard;
#[cfg(not(unix))]
impl SigintGuard {
    fn protect() -> Self { Self }
}

#[cfg(not(unix))]
fn block_sigint_on_current_thread() {}

// ───────────────────────────── PythonRuntime ──────────────────────────────

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

// ─────────────────────── 核心：在独立线程执行 Python 插件 ───────────────────────

fn run_python_plugin(
    plugin_path: &Path,
    plugin_name: &str,
    subscriber_ids: &HashMap<String, usize>,
) {
    use pyo3::prelude::*;
    use pyo3::types::PyDict;

    // ── Step 1: 屏蔽此线程的 SIGINT ──
    // SIGINT 只应投递到主线程，由 tokio 统一处理。
    // Python 线程屏蔽后不再收到 Ctrl+C，不会抛 KeyboardInterrupt。
    block_sigint_on_current_thread();

    // ── Step 2: 初始化 Python 解释器（全局仅一次）──
    PYTHON_INIT.call_once(|| {
        pyo3::prepare_freethreaded_python();
    });

    // ── Step 3: RAII guard 保护进程级 SIGINT handler ──
    // Python 代码（`import signal`、第三方库）会通过 sigaction() 覆盖
    // tokio/signal_hook 注册的 handler。Guard 保证执行完后恢复。
    let _sig_guard = SigintGuard::protect();

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

            // 调用 plugin_main()
            let main_fn = plugin_mod.getattr("plugin_main")?;
            info!("[python] 插件 {} 开始执行", plugin_name_owned);

            match main_fn.call0() {
                Ok(_) => {
                    info!("[python] 插件 {} 已正常退出", plugin_name_owned);
                }
                Err(e) => {
                    error!("[python] 插件 {} 异常: {}", plugin_name_owned, e);
                    return Err(e);
                }
            }
            Ok(())
        })
    }));

    // _sig_guard 在此 drop → 恢复 tokio 的 SIGINT handler（如果是最后一个 guard）

    match result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => error!("[python] 插件 {} Python 错误: {}", plugin_name, e),
        Err(e) => error!("[python] 插件 {} panic: {:?}", plugin_name, e),
    }
}
