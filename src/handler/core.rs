use serde_json::Value;
use crate::handler;
use crate::error::{Result, LNErr};
use crate::event::PostType;
use crate::message::Message;
use crate::event::MetaEvent;
use crate::notice::Notice;


pub fn handle(data: Value) -> Result<()>{
    let post_type = data.get("post_type")
        .and_then(|v| serde_json::from_value::<PostType>(v.clone()).ok())
        .ok_or(LNErr::EventParseError("Invalid post_type".to_string()))?;

    // println!("post_type: {}", post_type);
    println!(">>> [收到推送消息]");
    match serde_json::to_string_pretty(&data) {
        Ok(pretty) => println!("{}", pretty),
        Err(_) => println!("{:?}", data),
    }

    match post_type {
        PostType::MetaEvent => {
            let meta_event: MetaEvent = MetaEvent::new(data.clone());
            handler::event::handle_event(meta_event)?;
        },
        PostType::Message => {
            let message: Message = Message::new(data.clone());
            handler::message::handle_message(message)?;
        },
        PostType::Notice => {
            let notice: Notice = Notice::new(data.clone());
            handler::notice::handle_notice(notice)?;
        },
        _ => {
        }
    }

    Ok(())

}