//! 核心模块
//! 
//! 包含机器人的核心功能和组件。

pub mod driver;
pub mod task;
pub mod handler;
// pub mod message;
pub mod plugin_manager;
// pub mod plugin_registry;

pub use driver::Driver;
pub use task::Task;
pub use plugin_manager::PluginManager;