use crate::error::Result;
use crate::notice::{Notice, NoticeType};
use crate::plugin::dispatch_notice;
use tracing::{debug, warn};

#[cfg(feature = "napcat")]
pub fn handle_notice(notice: Notice) -> Result<()> {
    debug!("收到通知: {:?}", notice);

    match notice.notice_type {
        // 好友相关
        NoticeType::FriendAdd | NoticeType::FriendRecall | NoticeType::FriendPoke => {
            dispatch_notice(notice);
        }

        // 群管理相关
        NoticeType::GroupAdmin
        | NoticeType::GroupBan
        | NoticeType::GroupIncrease
        | NoticeType::GroupDecrease => {
            dispatch_notice(notice);
        }

        // 群消息相关
        NoticeType::GroupCard | NoticeType::GroupRecall | NoticeType::GroupUpload => {
            dispatch_notice(notice);
        }

        // 群荣誉和头衔
        NoticeType::GroupTitle | NoticeType::Honor => {
            dispatch_notice(notice);
        }

        // 精华消息
        NoticeType::Essence => {
            dispatch_notice(notice);
        }

        // 戳一戳和互动
        NoticeType::Poke | NoticeType::LuckyKing => {
            dispatch_notice(notice);
        }

        // 表情回应（NapCat 扩展）
        NoticeType::GroupMsgEmojiLike => {
            dispatch_notice(notice);
        }

        // 通用通知
        NoticeType::Notify => {
            dispatch_notice(notice);
        }

        NoticeType::Unknown => {
            warn!("未知通知类型: {:?}", notice);
            dispatch_notice(notice);
        }
    };

    Ok(())
}
