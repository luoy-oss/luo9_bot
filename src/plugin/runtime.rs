// src/plugin/runtime.rs
use std::collections::HashMap;

/// 插件运行时抽象
///
/// 所有插件类型（原生 DLL、Python、Java 等）统一实现此 trait，
/// 使宿主的分发层和管理层无需感知具体运行时类型。
pub trait PluginRuntime: Send {
    /// 启动插件，传入宿主预分配的 subscriber ID 映射
    fn start(&mut self, subscriber_ids: HashMap<String, usize>) -> Result<(), String>;

    /// 检查插件是否仍在运行
    fn is_alive(&self) -> bool;

    /// 优雅停止插件（取消订阅 + 等待退出）
    fn stop(&mut self) -> Result<(), String>;

    /// 强制停止插件（用于热重载前，带超时）
    fn force_stop(&mut self) -> Result<(), String>;

    /// 获取插件的 subscriber ID 映射
    fn subscriber_ids(&self) -> &HashMap<String, usize>;
}
