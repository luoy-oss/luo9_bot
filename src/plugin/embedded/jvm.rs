// src/plugin/embedded/jvm.rs
//! JVM 嵌入式运行时（JNI）
//!
//! 通过 JNI 嵌入 JVM，在宿主进程内执行 Java/Kotlin 插件。
//! Java SDK 通过 JNA 直接调用 luo9_core.dll FFI。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;
use tracing::{info, error};

use crate::plugin::runtime::PluginRuntime;

/// JVM 插件运行时
pub struct JvmRuntime {
    plugin_path: PathBuf,
    thread_handle: Option<JoinHandle<()>>,
    subscriber_ids: HashMap<String, usize>,
}

impl JvmRuntime {
    pub fn new(path: &Path) -> Result<Self, String> {
        Ok(Self {
            plugin_path: path.to_path_buf(),
            thread_handle: None,
            subscriber_ids: HashMap::new(),
        })
    }
}

impl PluginRuntime for JvmRuntime {
    fn start(&mut self, subscriber_ids: HashMap<String, usize>) -> Result<(), String> {
        self.subscriber_ids = subscriber_ids;

        let plugin_path = self.plugin_path.clone();

        self.thread_handle = Some(std::thread::spawn(move || {
            // TODO: JNI 创建 JVM
            // 1. JNI_CreateJavaVM()
            // 2. 设置 classpath 包含插件 JAR
            // 3. 查找插件主类 (plugin.toml 中的 entry)
            // 4. 调用 plugin_main() 静态方法
            info!("JVM 插件线程启动: {:?}", plugin_path);
            error!("JVM 运行时尚未实现");
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
        // TODO: 通知 JVM 退出
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
