use crate::message::Message;
use crate::message::MsgType;
use crate::error::Result;
use crate::error::LNErr;
use crate::plugin::dispatch_message;

use tracing::warn;

#[cfg(feature = "napcat")]
pub fn handle_message(message: Message) -> Result<()> {
    println!("message: {:?}", message);
    
    match message.message_type {
        MsgType::Private | MsgType::Group => {
            if message.message_type == MsgType::Group && message.group_id.is_none() {
                warn!("群聊消息缺少 group_id: {:?}", message);
                return Err(LNErr::InvalidMessage);
            }
            
            dispatch_message(message);
        }
        _ => {
            use tracing::warn;
            warn!("不支持的消息类型: {:?}", message.message_type);
            return Err(LNErr::UnknownMsgType);
        }
    }
    Ok(())
}