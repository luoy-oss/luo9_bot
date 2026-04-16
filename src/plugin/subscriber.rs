// src/plugin/subscriber.rs
use std::ffi::CString;
use std::sync::Arc;
use libloading::{Library, Symbol};
use tokio::task;
use tracing::{debug, error, info, warn};
use tokio::sync::broadcast;

use crate::message::{Message, MsgType};
use super::bus;
use crate::event::MetaEvent;
use crate::notice::Notice;
use super::data::PluginData;

/// 插件处理结果
#[derive(Debug)]
pub struct PluginProcessResult {
    pub plugin_id: usize,
    pub success: bool,
    pub duration: std::time::Duration,
    pub error: Option<String>,
}

/// 插件订阅者
pub struct PluginSubscriber {
    lib: Arc<Library>,
    plugin_id: usize,
    timeout_duration: std::time::Duration,
}

impl PluginSubscriber {
    pub fn new(lib: Arc<Library>, plugin_id: usize) -> Self {
        Self {
            lib,
            plugin_id,
            timeout_duration: std::time::Duration::from_secs(30),
        }
    }
    
    /// 启动订阅者，开始监听消息
    pub fn start(&self) {
        let lib = Arc::clone(&self.lib);
        let plugin_id = self.plugin_id;
        let timeout_duration = self.timeout_duration;
        let mut rx = bus::subscribe();
        
        task::spawn(async move {
            info!("插件 #{} 开始监听消息总线", plugin_id);
            
            loop {
                match rx.recv().await {
                    Ok(data) => {
                        debug!("插件 #{} 收到数据: {:?}", plugin_id, data.type_name());
                        
                        let lib_clone = Arc::clone(&lib);
                        let data_clone = data.clone();
                        
                        // 在 spawn_blocking 中处理 FFI 调用
                        let handle = task::spawn_blocking(move || {
                            Self::process_plugin_data(&lib_clone, plugin_id, data_clone)
                        });
                        
                        // 监控处理超时
                        let plugin_id_for_log = plugin_id;
                        task::spawn(async move {
                            match tokio::time::timeout(timeout_duration, handle).await {
                                Ok(Ok(result)) => {
                                    if !result.success {
                                        error!(
                                            "插件 #{} 处理失败: {:?}", 
                                            plugin_id_for_log, 
                                            result.error
                                        );
                                    }
                                }
                                Ok(Err(e)) => {
                                    error!("插件 #{} 任务执行失败: {:?}", plugin_id_for_log, e);
                                }
                                Err(_) => {
                                    error!("插件 #{} 处理超时 ({:?})", plugin_id_for_log, timeout_duration);
                                }
                            }
                        });
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("插件 #{} 消息总线已关闭，停止监听", plugin_id);
                        break;
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("插件 #{} 落后 {} 条消息，可能处理过慢", plugin_id, n);
                        continue;
                    }
                }
            }
        });
    }
    
    /// 处理插件数据
    fn process_plugin_data(lib: &Library, plugin_id: usize, data: PluginData) -> PluginProcessResult {
        let start = std::time::Instant::now();
        
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            Self::call_plugin_function(lib, plugin_id, &data)
        }));
        
        let duration = start.elapsed();
        
        match result {
            Ok(Ok(_)) => {
                debug!("插件 #{} 处理{}成功，耗时: {:?}", plugin_id, data.type_name(), duration);
                PluginProcessResult {
                    plugin_id,
                    success: true,
                    duration,
                    error: None,
                }
            }
            Ok(Err(e)) => {
                error!("插件 #{} 处理{}失败，耗时: {:?}, 错误: {}", plugin_id, data.type_name(), duration, e);
                PluginProcessResult {
                    plugin_id,
                    success: false,
                    duration,
                    error: Some(e),
                }
            }
            Err(_) => {
                error!("插件 #{} 处理{} panic，耗时: {:?}", plugin_id, data.type_name(), duration);
                PluginProcessResult {
                    plugin_id,
                    success: false,
                    duration,
                    error: Some("Plugin panicked".to_string()),
                }
            }
        }
    }
    
    /// 调用插件函数
    fn call_plugin_function(
        lib: &Library, 
        plugin_id: usize, 
        data: &PluginData
    ) -> Result<(), String> {
        match data {
            PluginData::Message(msg) => {
                Self::call_message_function(lib, plugin_id, msg)
            }
            PluginData::MetaEvent(event) => {
                Self::call_meta_event_function(lib, plugin_id, event)
            }
            PluginData::Notice(notice) => {
                Self::call_notice_function(lib, plugin_id, notice)
            }
        }
    }
    
    /// 调用消息处理函数
    fn call_message_function(
        lib: &Library, 
        plugin_id: usize, 
        msg: &Message
    ) -> Result<(), String> {
        let c_message = CString::new(&*msg.message)
            .map_err(|e| format!("CString conversion failed: {}", e))?;
        let c_ptr = c_message.as_ptr();
        
        unsafe {
            match msg.message_type {
                MsgType::Private => {
                    let func: Symbol<extern "C" fn(u64, *const i8)> = lib
                        .get(b"pmsg_process\0")
                        .map_err(|_| "pmsg_process not found".to_string())?;
                    func(msg.user_id, c_ptr);
                }
                MsgType::Group => {
                    let func: Symbol<extern "C" fn(u64, u64, *const i8)> = lib
                        .get(b"gmsg_process\0")
                        .map_err(|_| "gmsg_process not found".to_string())?;
                    func(msg.group_id.unwrap_or(0), msg.user_id, c_ptr);
                }
                MsgType::Other => {
                    debug!("插件 #{} 忽略 Other 类型消息", plugin_id);
                }
            }
        }
        
        Ok(())
    }
    
    /// 调用元事件处理函数
    fn call_meta_event_function(
        _lib: &Library, 
        plugin_id: usize, 
        event: &MetaEvent
    ) -> Result<(), String> {
        debug!("插件 #{} 收到元事件: {:?}", plugin_id, event.meta_event_type);
        
        Ok(())
    }
    
    /// 调用通知处理函数
    fn call_notice_function(
        _lib: &Library, 
        plugin_id: usize, 
        notice: &Notice
    ) -> Result<(), String> {
        debug!("插件 #{} 收到通知: {:?}", plugin_id, notice.notice_type);
        
        Ok(())
    }
}

