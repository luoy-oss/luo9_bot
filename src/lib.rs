//! 洛玖机器人 Rust 版本库
//! 
//! 这个库提供了洛玖机器人的核心功能，包括配置管理、消息处理、插件系统等。

pub mod config;
pub mod core;
// pub mod napcat;
pub mod plugins;
pub mod utils;

// 重导出常用组件，方便使用
pub use config::{Config, Value, load_config};
pub use core::{Driver, Task};
pub use plugins::{Plugin, PluginManager};

/// 版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 初始化日志系统
pub fn init_logger() {
    tracing_subscriber::fmt::init();
}