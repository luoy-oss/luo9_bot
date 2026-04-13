use crate::message::Message;
use crate::message::MsgType;
use crate::error::Result;
use crate::error::LNErr;
use crate::plugin::dispatch_pmsg;
use crate::plugin::dispatch_gmsg;



#[cfg(feature = "napcat")]
pub fn handle_message(message: Message) -> Result<()> {
    println!("message: {:?}", message);
    match message.message_type {
        MsgType::Private => {
            dispatch_pmsg(message.user_id, &message.message);
        }
        MsgType::Group => {
            if message.group_id.is_some() {
                dispatch_gmsg( message.group_id.unwrap(), message.user_id, &message.message);
            }
        }
        _ => {
            use tracing::warn;
            warn!("其他消息类型: {:?}", message.message_type);
            return Err(LNErr::UnknownMsgType);
        }
    }
    Ok(())
}