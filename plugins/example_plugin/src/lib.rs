//! Hello 插件
//! 
//! 这是一个简单的示例插件，用于演示如何使用 Luo9 机器人的插件系统。

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::{Result, anyhow};
use luo9_bot::config::Value;
use luo9_bot::core::plugin_manager::{Plugin, PluginMetadata};
use luo9_bot::core::message::{GroupMessage, PrivateMessage};
use luo9_bot::export_plugin;
use luo9_bot::api::ApiManager;

/// Hello 插件结构
pub struct HelloPlugin {
    metadata: PluginMetadata,
    config: Arc<Value>,
    api: ApiManager,
}

impl HelloPlugin {
    /// 创建一个新的 Hello 插件
    pub async fn new(config: Arc<Value>) -> Result<Self> {
        let metadata = PluginMetadata {
            name: "hello_plugin".to_string(),
            describe: "一个简单的示例插件，用于演示基本功能".to_string(),
            author: "Luo9".to_string(),
            version: "0.1.0".to_string(),
            message_types: vec![
                "group_message".to_string(),
                "private_message".to_string(),
                "group_poke".to_string(),
            ],
        };
        
        // 初始化 API - 处理错误转换
        let api = match ApiManager::new(config.clone()) {
            Ok(api) => api,
            Err(e) => return Err(anyhow!("API初始化失败: {}", e)),
        };
        
        Ok(Self {
            metadata,
            config,
            api,
        })
    }
}

#[async_trait]
impl Plugin for HelloPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    async fn handle_group_message(&self, message: &GroupMessage) -> Result<()> {
        // 简单的消息处理逻辑
        if message.content.contains("你好") || message.content.contains("hello") {
            println!("收到群 {} 中用户 {} 的问候", message.group_id, message.sender_id);
            
            match self.api.send_group_message(&message.group_id, "你好啊！我是洛玖机器人！").await {
                Ok(_) => println!("成功发送群消息"),
                Err(e) => println!("发送群消息失败: {}", e)
            }    
        }
        
        Ok(())
    }
    
    async fn handle_private_message(&self, message: &PrivateMessage) -> Result<()> {
        // 简单的私聊消息处理逻辑
        if message.content.contains("你好") || message.content.contains("hello") {
            println!("收到用户 {} 的私聊问候", message.sender_id);
            
            match self.api.send_private_msg(&message.sender_id, "你好啊！我是洛玖机器人！").await {
                Ok(_) => println!("成功发送私聊消息"),
                Err(e) => println!("发送私聊消息失败: {}", e)
            }
        }
        
        Ok(())
    }
    
    async fn handle_group_poke(&self, target_id: &str, user_id: &str, group_id: &str) -> Result<()> {
        // 处理戳一戳事件
        println!("用户 {} 在群 {} 中戳了 {}", user_id, group_id, target_id);
        
        // 如果戳的是机器人自己
        if target_id == self.config.bot_id.to_string() {
            println!("有人戳我，我要回应！");
            match self.api.send_group_message(group_id, "别戳我，我怕痒！").await {
                Ok(_) => println!("成功发送回应消息"),
                Err(e) => println!("发送回应消息失败: {}", e)
            }
        }
        
        Ok(())
    }
}

// 创建插件实例的异步函数
async fn create(config: Arc<Value>) -> Result<Box<dyn Plugin>> {
    let plugin = HelloPlugin::new(config).await?;
    Ok(Box::new(plugin))
}

// 导出插件创建函数
export_plugin!(create);