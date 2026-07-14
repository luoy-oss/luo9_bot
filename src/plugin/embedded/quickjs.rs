// src/plugin/embedded/quickjs.rs
//! QuickJS 嵌入式运行时
//!
//! 内嵌 QuickJS 引擎（~210KB，零外部依赖），执行 JavaScript 插件。
//! SDK 通过 QuickJS C FFI 绑定 luo9_core 函数。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;
use tracing::{info, error};

use crate::plugin::runtime::PluginRuntime;

/// QuickJS 插件运行时
pub struct QuickjsRuntime {
    plugin_path: PathBuf,
    thread_handle: Option<JoinHandle<()>>,
    subscriber_ids: HashMap<String, usize>,
}

impl QuickjsRuntime {
    pub fn new(path: &Path) -> Result<Self, String> {
        Ok(Self {
            plugin_path: path.to_path_buf(),
            thread_handle: None,
            subscriber_ids: HashMap::new(),
        })
    }
}

impl PluginRuntime for QuickjsRuntime {
    fn start(&mut self, subscriber_ids: HashMap<String, usize>) -> Result<(), String> {
        self.subscriber_ids = subscriber_ids;

        let plugin_path = self.plugin_path.clone();

        self.thread_handle = Some(std::thread::spawn(move || {
            // TODO: QuickJS 引擎初始化
            // 1. JS_NewRuntime() + JS_NewContext()
            // 2. 注册 luo9_core FFI 绑定到 JS 全局对象
            // 3. 加载并执行插件 .js 文件
            // 4. 调用 plugin_main()
            info!("QuickJS 插件线程启动: {:?}", plugin_path);
            error!("QuickJS 运行时尚未实现");
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
