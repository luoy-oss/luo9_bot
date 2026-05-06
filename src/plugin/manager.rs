// src/plugin/manager.rs
use std::collections::HashMap;
use tokio::sync::Mutex;
use tracing::{info, warn};

use super::handle::PluginHandle;

/// 插件信息（用于 API 返回和注册）
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub id: usize,
    pub name: String,
    pub version: String,
    pub enabled: bool,
    pub path: Option<String>,
    pub priority: i32,
    pub block_enabled: bool,
    pub active: bool,
}

/// 优先级分发条目
#[derive(Debug, Clone)]
pub struct DispatchEntry {
    pub name: String,
    pub priority: i32,
    pub block_enabled: bool,
    pub message_sub_id: Option<usize>,
    pub notice_sub_id: Option<usize>,
    pub meta_event_sub_id: Option<usize>,
}

/// 插件管理器
pub struct PluginManager {
    plugin_infos: Vec<PluginInfo>,
    handles: HashMap<String, PluginHandle>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugin_infos: Vec::new(),
            handles: HashMap::new(),
        }
    }

    /// 注册插件信息
    pub fn register_plugin(&mut self, info: PluginInfo) {
        info!("插件 #{} ({}) 已注册", info.id, info.name);
        self.plugin_infos.push(info);
    }

    /// 注册插件句柄
    pub fn register_handle(&mut self, handle: PluginHandle) {
        info!("插件 {} 句柄已注册", handle.name);
        self.handles.insert(handle.name.clone(), handle);
    }

    /// 获取所有插件信息
    pub fn get_all_plugins(&self) -> &[PluginInfo] {
        &self.plugin_infos
    }

    /// 按名称获取插件信息
    pub fn get_plugin_info(&self, name: &str) -> Option<&PluginInfo> {
        self.plugin_infos.iter().find(|p| p.name == name)
    }

    /// 按名称获取可变插件信息
    pub fn get_plugin_info_mut(&mut self, name: &str) -> Option<&mut PluginInfo> {
        self.plugin_infos.iter_mut().find(|p| p.name == name)
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

    /// 禁用插件（运行时热禁用）
    ///
    /// 1. 取消所有 topic 订阅（触发 sentinel，插件线程退出）
    /// 2. 等待线程退出（5 秒超时）
    /// 3. 标记为 inactive
    pub async fn disable_plugin(&mut self, name: &str) -> Result<String, String> {
        // 获取句柄
        let handle = self.handles.get_mut(name)
            .ok_or_else(|| format!("插件 {name} 不存在"))?;

        if !handle.active {
            return Err(format!("插件 {name} 已经是禁用状态"));
        }

        info!("正在禁用插件: {}", name);

        // 1. 取消所有订阅
        handle.unsubscribe_all();

        // 2. 等待线程退出
        let exited = handle.wait_exit(std::time::Duration::from_secs(5));
        if !exited {
            warn!("插件 {} 线程在 5 秒内未退出，标记为 inactive 但线程可能仍在运行", name);
        } else {
            info!("插件 {} 线程已退出", name);
        }

        // 3. 标记为 inactive
        handle.active = false;

        // 更新 PluginInfo
        if let Some(info) = self.plugin_infos.iter_mut().find(|p| p.name == name) {
            info.active = false;
        }

        Ok(format!("插件 {name} 已禁用"))
    }

    /// 获取优先级分发列表（按优先级降序）
    pub fn get_dispatch_list(&self) -> Vec<DispatchEntry> {
        let mut entries: Vec<DispatchEntry> = self.handles.values()
            .filter(|h| h.active)
            .map(|h| DispatchEntry {
                name: h.name.clone(),
                priority: h.priority,
                block_enabled: h.block_enabled,
                message_sub_id: h.subscriber_ids.get("luo9_message").copied(),
                notice_sub_id: h.subscriber_ids.get("luo9_notice").copied(),
                meta_event_sub_id: h.subscriber_ids.get("luo9_meta_event").copied(),
            })
            .collect();

        // 按优先级降序排序
        entries.sort_by(|a, b| b.priority.cmp(&a.priority));
        entries
    }

    /// 更新插件优先级
    pub fn update_priority(&mut self, name: &str, priority: i32) -> Result<(), String> {
        if let Some(handle) = self.handles.get_mut(name) {
            handle.priority = priority;
        }
        if let Some(info) = self.plugin_infos.iter_mut().find(|p| p.name == name) {
            info.priority = priority;
            Ok(())
        } else {
            Err(format!("插件 {name} 不存在"))
        }
    }

    /// 更新插件阻断设置
    pub fn update_block(&mut self, name: &str, block_enabled: bool) -> Result<(), String> {
        if let Some(handle) = self.handles.get_mut(name) {
            handle.block_enabled = block_enabled;
        }
        if let Some(info) = self.plugin_infos.iter_mut().find(|p| p.name == name) {
            info.block_enabled = block_enabled;
            Ok(())
        } else {
            Err(format!("插件 {name} 不存在"))
        }
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> String {
        let active = self.handles.values().filter(|h| h.active).count();
        format!("插件统计: 总数={}, 活跃={}", self.plugin_infos.len(), active)
    }
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_PLUGIN_MANAGER: Mutex<PluginManager> = {
        Mutex::new(PluginManager::new())
    };
}

/// 初始化全局插件管理器
///
/// `config_entries` 为配置文件中的插件条目，用于覆盖默认的 priority/block_enabled。
pub async fn init_global_manager(
    infos: Vec<PluginInfo>,
    handles: Vec<PluginHandle>,
    config_entries: &[crate::config::PluginEntry],
) {
    let mut manager = GLOBAL_PLUGIN_MANAGER.lock().await;
    for mut info in infos {
        // 从配置中应用 priority/block_enabled
        if let Some(entry) = config_entries.iter().find(|e| e.name == info.name) {
            info.priority = entry.priority;
            info.block_enabled = entry.block_enabled;
        }
        manager.register_plugin(info);
    }
    for mut handle in handles {
        // 从配置中应用 priority/block_enabled
        if let Some(entry) = config_entries.iter().find(|e| e.name == handle.name) {
            handle.priority = entry.priority;
            handle.block_enabled = entry.block_enabled;
        }
        manager.register_handle(handle);
    }
    info!("全局插件管理器初始化完成");
}

/// 获取插件管理器统计信息
pub async fn get_manager_stats() -> String {
    let manager = GLOBAL_PLUGIN_MANAGER.lock().await;
    manager.get_stats()
}
