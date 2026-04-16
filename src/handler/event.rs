// use crate::sub_type::SubType;
use crate::event::MetaEvent;
use crate::event::MetaEventType;
use crate::plugin::dispatch_meta_event;

use crate::error::{Result, LNErr};

// use tracing::info;
// use tracing::warn;


#[cfg(feature = "napcat")]
pub fn handle_event(meta_event: MetaEvent) -> Result<()> {
    println!("meta_event：{:?}", meta_event);
    match meta_event.meta_event_type {
        // MetaEventType::Heartbeat => {
        //     #[cfg(feature = "bot_debug")]
        //     info!("心跳维持: {:?}", meta_event);
        // },
        // MetaEventType::Lifecycle => {
        //     match meta_event.sub_type {
        //         SubType::Enable => {
        //             info!("OneBot 启用...");
        //         },
        //         SubType::Disable => {
        //             warn!("OneBot 禁用");
        //         },
        //         SubType::Connect => {
        //             info!("WebSocket 连接成功!");
        //         },
        //         _ => {
        //             use tracing::warn;
        //             warn!("其他生命周期子类型: {:?}", meta_event.sub_type);
        //             return Err(LNErr::UnknownMetaEventType);
        //         }
        //     }
        // },
        MetaEventType::Heartbeat | MetaEventType::Lifecycle => {
            dispatch_meta_event(meta_event);
        },
        _ => {
            use tracing::warn;
            warn!("不支持的事件类型: {:?}", meta_event.meta_event_type);
            return Err(LNErr::UnknownMetaEventType);
        }
    }

    Ok(())
}