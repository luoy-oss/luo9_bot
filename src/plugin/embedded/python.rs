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

/// Python 插件运行时
pub struct PythonRuntime {
    plugin_path: PathBuf,
    thread_handle: Option<JoinHandle<()>>,
    subscriber_ids: HashMap<String, usize>,
}

impl PythonRuntime {
    pub fn new(path: &Path) -> Result<Self, String> {
        Ok(Self {
            plugin_path: path.to_path_buf(),
            thread_handle: None,
            subscriber_ids: HashMap::new(),
        })
    }
}

impl PluginRuntime for PythonRuntime {
    fn start(&mut self, subscriber_ids: HashMap<String, usize>) -> Result<(), String> {
        self.subscriber_ids = subscriber_ids;

        let plugin_path = self.plugin_path.clone();

        self.thread_handle = Some(std::thread::spawn(move || {
            // TODO: PyO3 嵌入 CPython 解释器
            // 1. 获取 GIL
            // 2. 添加 luo9_sdk Python 包到 sys.path
            // 3. 加载插件 .py 文件
            // 4. 调用 plugin_main()
            info!("Python 插件线程启动: {:?}", plugin_path);
            error!("Python 运行时尚未实现");
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
        // TODO: 通知 Python 解释器退出
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
