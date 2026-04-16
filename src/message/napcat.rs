use serde_json::Value;
use tokio::runtime::Handle;


/*
{
  "font": 14,
  "group_id": 736215193,
  "message": [
    {
      "data": {
        "text": "eeeeeee"
      },
      "type": "text"
    }
  ],
  "message_format": "array",
  "message_id": 1471815977,
  "message_seq": 1471815977,
  "message_type": "group",
  "post_type": "message",
  "raw_message": "eeeeeee",
  "real_id": 1471815977,
  "real_seq": "46182",
  "self_id": 512166443,
  "sender": {
    "card": "[QQ红包] 恭喜发财",
    "nickname": "洛",
    "role": "admin",
    "user_id": 2557657882
  },
  "sub_type": "normal",
  "time": 1774857824,
  "user_id": 2557657882
}
*/


#[derive(Debug, Clone, PartialEq)]
pub enum MsgType {
    Private,
    Group,
    Other,
}


#[derive(Debug, Clone)]
pub struct Message {
    pub message_type: MsgType,
    pub user_id: u64,
    pub group_id: Option<u64>,
    pub message: String,
}

impl Message {
    pub fn new(data: Value) -> Self {
        let message_type = match data.get("message_type").and_then(|v| v.as_str()) {
            Some("private") => MsgType::Private,
            Some("group") => MsgType::Group,
            _ => MsgType::Other,
        };

        let user_id = data.get("user_id").and_then(|v| v.as_u64()).unwrap_or(0);

        let group_id = data.get("group_id").and_then(|v| v.as_u64());

        let message = data.get("raw_message").and_then(|v| v.as_str()).unwrap_or("").to_string();

        Self{
            message_type,
            user_id,
            group_id,
            message,
        }


    }
}

use std::ffi::{c_ulonglong, c_longlong, c_char};
use std::ffi::CStr;
use std::sync::OnceLock;
use crate::connection::Sender;
use crate::error::Result;
use tracing::{error, warn};

// 导入 core 的注册函数
unsafe extern "C" {
    pub(crate) unsafe fn luo9_register_send_group_msg(f: extern "C" fn(c_ulonglong, *const c_char) -> c_longlong);
    pub(crate) unsafe fn luo9_register_send_private_msg(f: extern "C" fn(c_ulonglong, *const c_char) -> c_longlong);
}

static GLOBAL_SENDER: OnceLock<Sender> = OnceLock::new();

pub fn init_global_sender(sender: Sender) -> Result<&'static str> {
    let _ = GLOBAL_SENDER.set(sender);
    Ok("全局 Sender 实例已初始化")
}

pub fn get_global_sender() -> Option<&'static Sender> {
    GLOBAL_SENDER.get()
}

pub(crate) extern "C" fn real_send_group_msg(group_id: c_ulonglong, msg: *const c_char) -> c_longlong {
    let msg_str = unsafe { CStr::from_ptr(msg).to_string_lossy() };

    let Some(sender) = get_global_sender().cloned() else {
        error!("⚠ 全局 Sender 未初始化");
        return -2;
    };

    let group_id = group_id as u64;
    let msg_str = msg_str.to_string();
    
    Handle::current().spawn(async move {
        if let Err(e) = sender.send_group_message(group_id, &msg_str).await {
            warn!("✗ 群消息发送失败: {}", e);
        }
    });

    10086
}

pub(crate) extern "C" fn real_send_private_msg(user_id: c_ulonglong, msg: *const c_char) -> c_longlong {
    let msg_str = unsafe { CStr::from_ptr(msg).to_string_lossy() };

    let Some(sender) = get_global_sender().cloned() else {
        error!("⚠ 全局 Sender 未初始化");
        return -2;
    };

    let user_id = user_id as u64;
    let msg_str = msg_str.to_string();
    
    Handle::current().spawn(async move {
        if let Err(e) = sender.send_private_message(user_id, &msg_str).await {
            warn!("✗ 私聊发送失败: {}", e);
        }
    });

    10087
}