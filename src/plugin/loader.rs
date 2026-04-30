// src/plugin/loader.rs
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use libloading::Library;
use tracing::{info, error, warn};

use super::manager::PluginInfo;

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

    /// 加载所有插件，返回插件信息列表
    /// 加载后为每个插件启动独立线程调用 plugin_main()
    pub fn load_all(&self) -> Result<Vec<PluginInfo>, String> {
        let mut infos = Vec::new();

        if !self.plugin_dir.exists() {
            fs::create_dir_all(&self.plugin_dir)
                .map_err(|e| format!("创建插件目录失败: {}", e))?;
            info!("已创建插件目录: {:?}", self.plugin_dir);
            return Ok(infos);
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
                Ok(info) => {
                    info!("成功加载插件: {} (ID: {})", info.name, info.id);
                    infos.push(info);
                }
                Err(e) => {
                    error!("加载插件失败 {:?}: {}", path, e);
                }
            }
        }

        Ok(infos)
    }

    /// 加载单个插件并启动其 plugin_main 线程
    fn load_single(&self, path: &Path, default_id: usize) -> Result<PluginInfo, String> {
        unsafe {
            let lib = Arc::new(
                Library::new(path)
                    .map_err(|e| format!("加载动态库失败: {}", e))?
            );

            // 检查是否导出了 plugin_main
            let has_main = lib.get::<unsafe extern "C" fn()>(b"plugin_main\0").is_ok();

            let info = PluginInfo {
                id: default_id,
                name: path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                version: String::new(),
                enabled: has_main,
                path: Some(path.to_string_lossy().to_string()),
            };

            if has_main {
                let lib_clone = Arc::clone(&lib);
                let plugin_name = info.name.clone();
                std::thread::spawn(move || {
                    Self::run_plugin(lib_clone, &plugin_name);
                });
            } else {
                warn!("插件 {} 未导出 plugin_main，跳过", info.name);
            }

            Ok(info)
        }
    }

    /// 在独立线程中驱动插件的 plugin_main 循环
    ///
    /// 插件运行在独立线程中，panic 被捕获后不会影响主程序。
    /// 注意：段错误（segfault）等硬件异常无法捕获，这是 DLL 插件模型的固有限制。
    fn run_plugin(lib: Arc<Library>, plugin_name: &str) {
        unsafe {
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
}
