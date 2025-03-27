//! 消息模块
//! 
//! 这个模块定义了不同类型的消息结构。

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// 基础消息特性
pub trait Message {
    /// 获取消息ID
    fn message_id(&self) -> &str;
    
    /// 获取消息内容
    fn content(&self) -> &str;
    
    /// 获取发送者ID
    fn sender_id(&self) -> &str;
}

/// 群消息结构
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GroupMessage {
    pub message_id: String,
    pub content: String,
    pub sender_id: String,
    pub group_id: String,
    pub raw_data: JsonValue,
}

impl Message for GroupMessage {
    fn message_id(&self) -> &str {
        &self.message_id
    }
    
    fn content(&self) -> &str {
        &self.content
    }
    
    fn sender_id(&self) -> &str {
        &self.sender_id
    }
}

/// 私聊消息结构
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PrivateMessage {
    pub message_id: String,
    pub content: String,
    pub sender_id: String,
    pub raw_data: JsonValue,
}

impl Message for PrivateMessage {
    fn message_id(&self) -> &str {
        &self.message_id
    }
    
    fn content(&self) -> &str {
        &self.content
    }
    
    fn sender_id(&self) -> &str {
        &self.sender_id
    }
}