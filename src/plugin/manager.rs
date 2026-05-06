// src/plugin/manager.rs
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::sync::Mutex;
use tracing::{info, warn, error};

use super::handle::PluginHandle;
use super::loader::load_single_plugin;
use super::dispatch::update_dispatch_list;

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
    /// 4. 从 handles 中移除（释放 Arc<Library>，解锁 DLL 文件）
    /// 5. 更新分发列表
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

        // 4. 从 handles 中移除（释放 Arc<Library>，解锁 DLL 文件）
        self.handles.remove(name);

        // 5. 更新分发列表
        let entries = self.get_dispatch_list();
        update_dispatch_list(entries);

        Ok(format!("插件 {name} 已禁用"))
    }

    /// 启用插件（运行时热加载）
    ///
    /// 1. 加载 DLL
    /// 2. 创建 subscriber
    /// 3. spawn plugin_main 线程
    /// 4. 注册 PluginHandle 和 PluginInfo
    /// 5. 更新分发列表
    pub async fn enable_plugin(
        &mut self,
        name: &str,
        path: &Path,
        config_entries: &[crate::config::PluginEntry],
    ) -> Result<String, String> {
        // 检查是否已有同名且 active 的插件
        if let Some(handle) = self.handles.get(name) {
            if handle.active {
                return Err(format!("插件 {name} 已经在运行中"));
            }
        }

        // 移除 inactive 的旧句柄（释放 Arc<Library>）
        if let Some(old) = self.handles.remove(name) {
            if old.active {
                // 不应该到这里，但以防万一
                return Err(format!("插件 {name} 仍在运行中"));
            }
            info!("已移除插件 {} 的旧句柄", name);
        }

        // 也移除旧的 PluginInfo
        self.plugin_infos.retain(|p| p.name != name);

        // 加载插件
        let next_id = self.plugin_infos.len();
        let (mut info, handle_opt) = load_single_plugin(path, next_id)
            .map_err(|e| format!("加载插件 {name} 失败: {e}"))?;

        let Some(mut handle) = handle_opt else {
            return Err(format!("插件 {name} 未导出 plugin_main"));
        };

        // 从配置中应用 priority/block_enabled
        if let Some(entry) = config_entries.iter().find(|e| e.name == name) {
            info.priority = entry.priority;
            info.block_enabled = entry.block_enabled;
            handle.priority = entry.priority;
            handle.block_enabled = entry.block_enabled;
        }

        // 注册
        self.register_plugin(info);
        self.register_handle(handle);

        // 更新分发列表
        let entries = self.get_dispatch_list();
        update_dispatch_list(entries);

        info!("插件 {} 已启用并加载", name);
        Ok(format!("插件 {name} 已启用并加载"))
    }

    /// 热重载插件（禁用后重新加载）
    ///
    /// 1. 获取旧句柄的 path
    /// 2. 禁用插件
    /// 3. 重新加载
    pub async fn reload_plugin(
        &mut self,
        name: &str,
        config_entries: &[crate::config::PluginEntry],
    ) -> Result<String, String> {
        // 获取旧句柄的 path
        let path = self.handles.get(name)
            .map(|h| h.path.clone())
            .or_else(|| {
                self.plugin_infos.iter()
                    .find(|p| p.name == name)
                    .and_then(|p| p.path.as_ref().map(|s| PathBuf::from(s)))
            })
            .ok_or_else(|| format!("插件 {name} 不存在，无法获取路径"))?;

        // 禁用（如果 active）
        if let Some(handle) = self.handles.get(name) {
            if handle.active {
                match self.disable_plugin(name).await {
                    Ok(_) => {}
                    Err(e) => {
                        error!("禁用插件 {} 失败: {}", name, e);
                        return Err(format!("禁用插件 {name} 失败: {e}"));
                    }
                }
            }
        }

        // 确保旧句柄已移除
        self.handles.remove(name);
        self.plugin_infos.retain(|p| p.name != name);

        // 重新加载
        self.enable_plugin(name, &path, config_entries).await
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
