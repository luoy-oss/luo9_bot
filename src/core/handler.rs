//! 消息处理模块
//! 
//! 负责处理接收到的消息和通知事件。

use std::sync::Arc;
use anyhow::Result;
use serde_json::Value as JsonValue;
use crate::config::Value;
use crate::core::message::{GroupMessage, PrivateMessage};
use crate::core::plugin_manager::PluginManager;
// use crate::api::APIManager;

/// 消息结构体
#[derive(Debug, Clone)]
pub struct Message {
    /// 消息类型 (private/group)
    pub message_type: String,
    /// 消息内容
    pub content: String,
    /// 发送者ID
    pub user_id: u64,
    /// 群组ID (如果是群消息)
    pub group_id: Option<u64>,
    /// 原始消息数据
    pub raw_data: JsonValue,
}

/// 处理消息事件
pub async fn message_handle(_value: &Arc<Value>, data: &JsonValue, plugin_manager: &PluginManager) -> Result<()> {
    let message_type = data["message_type"].as_str().unwrap_or("unknown");
    let content = data["message"].as_str().unwrap_or("");
    let user_id = data["user_id"].as_u64().unwrap_or(0);
    
    // tracing::info!(
    //     "收到{}消息: {} (来自: {})",
    //     message_type,
    //     content,
    //     user_id
    // );
    
    // 构建消息对象
    let message = Message {
        message_type: message_type.to_string(),
        content: content.to_string(),
        user_id,
        group_id: data["group_id"].as_u64(),
        raw_data: data.clone(),
    };
    
    // 创建API实例
    
    // 根据消息类型处理
    match message_type {
        "private" => {
            // 处理私聊消息
            let private_message = PrivateMessage {
                message_id: data["message_id"].as_str().unwrap_or("").to_string(),
                content: content.to_string(),
                sender_id: user_id.to_string(),
                raw_data: data.clone(),
            };
            
            plugin_manager.handle_private_message(&private_message).await?;
        }
        "group" => {
            // 处理群消息
            let group_id = match message.group_id {
                Some(id) => id,
                None => {
                    tracing::error!("群消息缺少群ID");
                    return Ok(());
                }
            };
            
            let group_message = GroupMessage {
                message_id: data["message_id"].as_str().unwrap_or("").to_string(),
                content: content.to_string(),
                sender_id: user_id.to_string(),
                group_id: group_id.to_string(),
                raw_data: data.clone(),
            };
            
            plugin_manager.handle_group_message(&group_message).await?;
        }
        _ => {
            tracing::warn!("未知的消息类型: {}", message_type);
        }
    }
    
    Ok(())
}

/// 处理通知事件
pub async fn notice_handle(value: &Arc<Value>, data: &JsonValue, plugin_manager: &PluginManager) -> Result<()> {
    let notice_type = data["notice_type"].as_str().unwrap_or("unknown");
    
    // tracing::info!("收到通知: {}", notice_type);
    
    // 根据通知类型处理
    match notice_type {
        "group_increase" => {
            // 处理群成员增加通知
            handle_group_increase(value, data).await?;
        }
        "friend_add" => {
            // 处理好友添加通知
            handle_friend_add(value, data).await?;
        }
        "notify" => {
            // 处理通知事件
            let sub_type = data["sub_type"].as_str().unwrap_or("");
            if sub_type == "poke" {
                let target_id = data["target_id"].as_u64().unwrap_or(0).to_string();
                let user_id = data["user_id"].as_u64().unwrap_or(0).to_string();
                let group_id = data["group_id"].as_u64().unwrap_or(0).to_string();
                
                plugin_manager.handle_group_poke(&target_id, &user_id, &group_id).await?;
            }
        }
        _ => {
            tracing::debug!("未处理的通知类型: {}", notice_type);
        }
    }
    
    Ok(())
}

/// 处理群成员增加通知
async fn handle_group_increase(_value: &Arc<Value>, data: &JsonValue) -> Result<()> {
    // 这里实现群成员增加的处理逻辑
    let group_id = data["group_id"].as_u64().unwrap_or(0);
    let user_id = data["user_id"].as_u64().unwrap_or(0);
    
    tracing::info!("用户 {} 加入群 {}", user_id, group_id);
    
    // 创建API实例
    // let api = APIManager::new(value.clone()).await?;
    
    // // 发送欢迎消息
    // api.send_group_message(
    //     &group_id.to_string(),
    //     &format!("欢迎 [CQ:at,qq={}] 加入本群！", user_id)
    // ).await?;
    
    Ok(())
}

/// 处理好友添加通知
async fn handle_friend_add(_value: &Arc<Value>, data: &JsonValue) -> Result<()> {
    // 这里实现好友添加的处理逻辑
    let user_id = data["user_id"].as_u64().unwrap_or(0);
    
    tracing::info!("用户 {} 已添加为好友", user_id);
    
    // 创建API实例
    // let api = APIManager::new(value.clone()).await?;
    
    // // 发送欢迎消息
    // api.send_private_msg(
    //     &user_id.to_string(),
    //     "你好，感谢添加我为好友！我是洛玖机器人~"
    // ).await?;
    
    Ok(())
}