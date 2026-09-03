// src/plugin/loader.rs
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, error, warn};

use super::handle::PluginHandle;
use super::manager::{PluginInfo, PluginStats};
use super::bus::{Bus, TOPIC_MESSAGE, TOPIC_NOTICE, TOPIC_META_EVENT, TOPIC_REQUEST, TOPIC_TASK, TOPIC_SEND};
use super::native_runtime::NativeRuntime;
use super::runtime::PluginRuntime;

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

    /// 加载单个插件
    fn load_single(&self, path: &Path, default_id: usize) -> Result<(PluginInfo, Option<PluginHandle>), String> {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        match ext {
            "dll" | "so" => self.load_native_plugin(path, default_id),
            "py" => self.load_python_plugin(path, default_id),
            "jar" => self.load_jvm_plugin(path, default_id),
            "js" => self.load_quickjs_plugin(path, default_id),
            _ => {
                warn!("不支持的插件类型: {:?}", path);
                Ok((Self::make_info(path, default_id, false), None))
            }
        }
    }

    /// 加载原生 DLL/SO 插件
    fn load_native_plugin(&self, path: &Path, default_id: usize) -> Result<(PluginInfo, Option<PluginHandle>), String> {
        unsafe {
            let mut runtime = NativeRuntime::new(&path.to_path_buf())?;
            let has_main = runtime.has_plugin_main();

            let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("unknown");
            let plugin_name = super::native_runtime::extract_display_name(file_name);

            let info = PluginInfo {
                id: default_id,
                name: plugin_name.clone(),
                version: String::new(),
                enabled: has_main,
                path: Some(path.to_string_lossy().to_string()),
                priority: 0,
                block_enabled: false,
                active: has_main,
                stats: PluginStats::default(),
            };

            let handle = if has_main {
                let subscriber_ids = Self::create_subscribers(&plugin_name);
                runtime.start(subscriber_ids.clone())?;

                Some(PluginHandle {
                    name: plugin_name,
                    runtime: Box::new(runtime),
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

    /// 加载 Python 插件
    fn load_python_plugin(&self, path: &Path, default_id: usize) -> Result<(PluginInfo, Option<PluginHandle>), String> {
        #[cfg(feature = "python-plugin")]
        {
            let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("unknown");
            let plugin_name = file_name.trim_end_matches(".py").to_string();

            let subscriber_ids = Self::create_subscribers(&plugin_name);

            let mut runtime = super::embedded::python::PythonRuntime::new(path)?;
            runtime.start(subscriber_ids)?;

            let info = PluginInfo {
                id: default_id,
                name: plugin_name.clone(),
                version: String::new(),
                enabled: true,
                path: Some(path.to_string_lossy().to_string()),
                priority: 0,
                block_enabled: false,
                active: true,
                stats: PluginStats::default(),
            };

            Ok((info, Some(PluginHandle {
                name: plugin_name,
                runtime: Box::new(runtime),
                priority: 0,
                block_enabled: false,
                active: true,
                path: path.to_path_buf(),
            })))
        }

        #[cfg(not(feature = "python-plugin"))]
        {
            warn!("Python 插件支持未启用（需启用 python-plugin feature）: {:?}", path);
            Ok((Self::make_info(path, default_id, false), None))
        }
    }

    /// 加载 JVM 插件
    fn load_jvm_plugin(&self, path: &Path, default_id: usize) -> Result<(PluginInfo, Option<PluginHandle>), String> {
        #[cfg(feature = "java-plugin")]
        {
            let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("unknown");
            let plugin_name = file_name.trim_end_matches(".jar").to_string();

            let subscriber_ids = Self::create_subscribers(&plugin_name);

            let mut runtime = super::embedded::jvm::JvmRuntime::new(path)?;
            runtime.start(subscriber_ids)?;

            let info = PluginInfo {
                id: default_id,
                name: plugin_name.clone(),
                version: String::new(),
                enabled: true,
                path: Some(path.to_string_lossy().to_string()),
                priority: 0,
                block_enabled: false,
                active: true,
                stats: PluginStats::default(),
            };

            Ok((info, Some(PluginHandle {
                name: plugin_name,
                runtime: Box::new(runtime),
                priority: 0,
                block_enabled: false,
                active: true,
                path: path.to_path_buf(),
            })))
        }

        #[cfg(not(feature = "java-plugin"))]
        {
            warn!("Java 插件支持未启用（需启用 java-plugin feature）: {:?}", path);
            Ok((Self::make_info(path, default_id, false), None))
        }
    }

    /// 加载 QuickJS 插件
    fn load_quickjs_plugin(&self, path: &Path, default_id: usize) -> Result<(PluginInfo, Option<PluginHandle>), String> {
        #[cfg(feature = "quickjs-plugin")]
        {
            let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("unknown");
            let plugin_name = file_name.trim_end_matches(".js").to_string();

            let subscriber_ids = Self::create_subscribers(&plugin_name);

            let mut runtime = super::embedded::quickjs::QuickjsRuntime::new(path)?;
            runtime.start(subscriber_ids)?;

            let info = PluginInfo {
                id: default_id,
                name: plugin_name.clone(),
                version: String::new(),
                enabled: true,
                path: Some(path.to_string_lossy().to_string()),
                priority: 0,
                block_enabled: false,
                active: true,
                stats: PluginStats::default(),
            };

            Ok((info, Some(PluginHandle {
                name: plugin_name,
                runtime: Box::new(runtime),
                priority: 0,
                block_enabled: false,
                active: true,
                path: path.to_path_buf(),
            })))
        }

        #[cfg(not(feature = "quickjs-plugin"))]
        {
            warn!("QuickJS 插件支持未启用（需启用 quickjs-plugin feature）: {:?}", path);
            Ok((Self::make_info(path, default_id, false), None))
        }
    }

    /// 为插件在各 topic 上创建 subscriber
    pub(crate) fn create_subscribers(plugin_name: &str) -> HashMap<String, usize> {
        info!("[loader] 为插件 {} 创建 subscriber...", plugin_name);
        let topics = [TOPIC_MESSAGE, TOPIC_NOTICE, TOPIC_META_EVENT, TOPIC_REQUEST, TOPIC_TASK, TOPIC_SEND];
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

    fn is_plugin_file(path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }

        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match ext {
            "dll" | "so" => true,
            "py" => cfg!(feature = "python-plugin"),
            "jar" => cfg!(feature = "java-plugin"),
            "js" => cfg!(feature = "quickjs-plugin"),
            _ => false,
        }
    }

    fn make_info(path: &Path, default_id: usize, enabled: bool) -> PluginInfo {
        let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("unknown");
        let plugin_name = file_name.to_string();
        PluginInfo {
            id: default_id,
            name: plugin_name,
            version: String::new(),
            enabled,
            path: Some(path.to_string_lossy().to_string()),
            priority: 0,
            block_enabled: false,
            active: false,
            stats: PluginStats::default(),
        }
    }

    /// 重新加载单个插件
    pub fn reload_single(&self, path: &Path, id: usize) -> Result<(PluginInfo, Option<PluginHandle>), String> {
        self.load_single(path, id)
    }
}

/// 独立的单个插件加载函数（供 enable_plugin / reload_plugin 调用）
pub fn load_single_plugin(path: &Path, default_id: usize) -> Result<(PluginInfo, Option<PluginHandle>), String> {
    let loader = PluginLoader::new(path.parent().unwrap_or(Path::new(".")));
    loader.load_single(path, default_id)
}
