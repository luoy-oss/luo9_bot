// src/plugin/manager.rs
use tokio::sync::Mutex;
use tracing::info;

/// 插件信息
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub id: usize,
    pub name: String,
    pub version: String,
    pub enabled: bool,
    pub path: Option<String>,
}

/// 插件管理器
pub struct PluginManager {
    plugin_infos: Vec<PluginInfo>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugin_infos: Vec::new(),
        }
    }

    /// 注册插件信息
    pub fn register_plugin(&mut self, info: PluginInfo) {
        info!("插件 #{} ({}) 已注册", info.id, info.name);
        self.plugin_infos.push(info);
    }

    /// 获取所有插件信息
    pub fn get_all_plugins(&self) -> &[PluginInfo] {
        &self.plugin_infos
    }

    /// 按名称更新插件版本
    pub fn update_plugin_version(&mut self, name: &str, version: &str) {
        if let Some(info) = self.plugin_infos.iter_mut().find(|p| p.name == name) {
            info.version = version.to_string();
            info!("插件 #{} ({}) 版本更新为: {}", info.id, info.name, version);
        }
    }

    /// 将所有空版本的插件标记为 Unknown
    pub fn mark_unknown_versions(&mut self) {
        for info in &mut self.plugin_infos {
            if info.version.is_empty() {
                info.version = "Unknown".to_string();
            }
        }
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> String {
        format!("插件统计: 总数={}", self.plugin_infos.len())
    }
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_PLUGIN_MANAGER: Mutex<PluginManager> = {
        Mutex::new(PluginManager::new())
    };
}

/// 初始化全局插件管理器
pub async fn init_global_manager(infos: Vec<PluginInfo>) {
    let mut manager = GLOBAL_PLUGIN_MANAGER.lock().await;
    for info in infos {
        manager.register_plugin(info);
    }
    info!("全局插件管理器初始化完成");
}

/// 获取插件管理器统计信息
pub async fn get_manager_stats() -> String {
    let manager = GLOBAL_PLUGIN_MANAGER.lock().await;
    manager.get_stats()
}
