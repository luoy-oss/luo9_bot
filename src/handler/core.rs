use serde_json::Value;
use crate::handler;
use crate::error::{Result, LNErr};
use crate::event::PostType;
use crate::message::Message;
use crate::event::MetaEvent;
use crate::notice::Notice;
use crate::request::Request;

use tracing::debug;


pub fn handle(data: Value) -> Result<()> {
    let post_type = data.get("post_type")
        .and_then(|v| serde_json::from_value::<PostType>(v.clone()).ok())
        .ok_or(LNErr::EventParseError("Invalid post_type".to_string()))?;

    // debug!("收到事件: {:?}", post_type);

    match post_type {
        PostType::MetaEvent => {
            let meta_event: MetaEvent = MetaEvent::new(data.clone());
            handler::event::handle_event(meta_event)?;
        },
        PostType::Message | PostType::MessageSent => {
            let message: Message = Message::new(data.clone());
            handler::message::handle_message(message)?;
        },
        PostType::Notice => {
            let notice: Notice = Notice::new(data.clone());
            handler::notice::handle_notice(notice)?;
        },
        PostType::Request => {
            let request: Request = Request::new(data.clone());
            handler::request::handle_request(request)?;
        },
        _ => {
            debug!("未处理的事件类型: {:?}", post_type);
        }
    }

    Ok(())

}