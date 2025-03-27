//! 插件系统模块
//! 
//! 提供插件的加载、管理和执行功能。

pub mod loader;
pub mod manager;

pub use loader::PluginLoader;
pub use manager::PluginManager;

use anyhow::Result;
use serde_json::Value as JsonValue;

/// 插件特性
pub trait Plugin: Send + Sync {
    /// 获取插件名称
    fn name(&self) -> &str;
    
    /// 获取插件描述
    fn description(&self) -> &str;
    
    /// 获取插件作者
    fn author(&self) -> &str;
    
    /// 插件加载时调用
    fn on_load(&self) -> Result<()>;
    
    /// 插件卸载时调用
    fn on_unload(&self) -> Result<()>;
    
    /// 处理消息
    fn on_message(&self, message_type: &str, data: &JsonValue) -> Result<bool>;
    
    /// 处理通知
    fn on_notice(&self, notice_type: &str, data: &JsonValue) -> Result<bool>;
}