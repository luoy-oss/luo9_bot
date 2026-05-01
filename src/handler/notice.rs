// use crate::sub_type::SubType;
use crate::plugin::dispatch_notice;

use crate::error::Result;
use crate::error::LNErr;

use crate::notice::Notice;
use crate::notice::NoticeType;

use tracing::warn;


#[cfg(feature = "napcat")]
pub fn handle_notice(notice: Notice) -> Result<()> {
    println!("notice: {:?}", notice);
    match notice.notice_type {
        // NoticeType::Notify => {
        //     match notice.sub_type {
        //         SubType::InputStatus => {
        //             #[cfg(feature = "bot_debug")]
        //             info!("输入状态通知: {:?}", notice.status);
        //         },
        //         _ => {
        //             warn!("其他通知子类型: {:?}", notice.sub_type);
        //             return Err(LNErr::UnknownNoticeType);
        //         }
        //     }
        // },
        NoticeType::Notify => {
            dispatch_notice(notice);
        },
        _ => {
            warn!("不支持的通知类型: {:?}", notice.notice_type);
            return Err(LNErr::UnknownNoticeType);
        }
    };

    Ok(())
}