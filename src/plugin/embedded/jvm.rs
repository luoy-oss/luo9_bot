// src/plugin/embedded/jvm.rs
//! JVM 嵌入式运行时（JNI）
//!
//! 通过 JNI 嵌入 JVM，在宿主进程内执行 Java/Kotlin 插件。
//! Java SDK 通过 JNA 直接调用 luo9_core.dll FFI。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;
use tracing::{error, info};

use crate::plugin::runtime::PluginRuntime;

/// JVM 插件运行时
pub struct JvmRuntime {
    plugin_path: PathBuf,
    plugin_name: String,
    thread_handle: Option<JoinHandle<()>>,
    subscriber_ids: HashMap<String, usize>,
}

impl JvmRuntime {
    pub fn new(path: &Path) -> Result<Self, String> {
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        let name = file_name.trim_end_matches(".jar").to_string();

        Ok(Self {
            plugin_path: path.to_path_buf(),
            plugin_name: name,
            thread_handle: None,
            subscriber_ids: HashMap::new(),
        })
    }
}

impl PluginRuntime for JvmRuntime {
    fn start(&mut self, subscriber_ids: HashMap<String, usize>) -> Result<(), String> {
        self.subscriber_ids = subscriber_ids;

        let plugin_path = self.plugin_path.clone();
        let plugin_name = self.plugin_name.clone();
        let subs = self.subscriber_ids.clone();

        self.thread_handle = Some(std::thread::spawn(move || {
            run_jvm_plugin(&plugin_path, &plugin_name, &subs);
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

/// 在独立线程中执行 JVM 插件
fn run_jvm_plugin(plugin_path: &Path, plugin_name: &str, _subscriber_ids: &HashMap<String, usize>) {
    use jni::{InitArgsBuilder, JavaVM};

    // 构建 classpath：插件 JAR + SDK JAR + 工作目录
    let plugin_jar = plugin_path.to_string_lossy();
    let cwd = std::env::current_dir().unwrap_or_default();
    let sdk_jar_dir = cwd.join("sdk").join("java").join("target");

    // 收集所有 JAR 文件作为 classpath
    let mut classpath_parts: Vec<String> = vec![plugin_jar.to_string()];

    // SDK JAR（如果有编译产物）
    if sdk_jar_dir.exists()
        && let Ok(entries) = std::fs::read_dir(&sdk_jar_dir)
    {
        classpath_parts.extend(
            entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| path.extension().is_some_and(|ext| ext == "jar"))
                .map(|path| path.to_string_lossy().to_string()),
        );
    }

    // 插件同目录下的其他 JAR（依赖库）
    if let Some(parent) = plugin_path.parent()
        && let Ok(entries) = std::fs::read_dir(parent)
    {
        classpath_parts.extend(
            entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| path != plugin_path)
                .filter(|path| path.extension().is_some_and(|ext| ext == "jar"))
                .map(|path| path.to_string_lossy().to_string()),
        );
    }

    let separator = if cfg!(windows) { ";" } else { ":" };
    let classpath = classpath_parts.join(separator);
    let classpath_opt = format!("-Djava.class.path={}", classpath);
    info!("[jvm] classpath: {}", classpath);

    // 创建 JVM
    let jvm_args = match InitArgsBuilder::new().option(&classpath_opt).build() {
        Ok(args) => args,
        Err(e) => {
            error!("[jvm] 构建 JVM 参数失败: {}", e);
            return;
        }
    };

    let jvm = match JavaVM::new(jvm_args) {
        Ok(vm) => vm,
        Err(e) => {
            error!("[jvm] 创建 JVM 失败: {}", e);
            return;
        }
    };

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut env = match jvm.attach_current_thread() {
            Ok(env) => env,
            Err(e) => {
                error!("[jvm] 附加线程到 JVM 失败: {}", e);
                return;
            }
        };

        // 查找插件主类（默认为 com.luo9.plugin.PluginMain）
        let class_name = "com/luo9/plugin/PluginMain";
        let main_class = match env.find_class(class_name) {
            Ok(cls) => cls,
            Err(e) => {
                error!("[jvm] 找不到插件主类 {}: {}", class_name, e);
                return;
            }
        };

        // 查找 plugin_main() 静态方法
        let main_method = match env.get_static_method_id(&main_class, "plugin_main", "()V") {
            Ok(method) => method,
            Err(e) => {
                error!("[jvm] 找不到 plugin_main() 方法: {}", e);
                return;
            }
        };

        info!("[jvm] 插件 {} 开始执行", plugin_name);

        // 调用 plugin_main()
        unsafe {
            match env.call_static_method_unchecked(
                &main_class,
                main_method,
                jni::signature::ReturnType::Primitive(jni::signature::Primitive::Void),
                &[],
            ) {
                Ok(_) => info!("[jvm] 插件 {} 已正常退出", plugin_name),
                Err(e) => error!("[jvm] 插件 {} 执行异常: {}", plugin_name, e),
            }
        }
    }));

    match result {
        Ok(()) => {}
        Err(e) => error!("[jvm] 插件 {} panic: {:?}", plugin_name, e),
    }
}
