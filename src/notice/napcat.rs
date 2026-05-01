use serde::{Deserialize, Serialize, Serializer};
use crate::sub_type::SubType;
use serde_json::Value;

#[derive(Debug, Clone)]
pub enum NoticeType {
    FriendAdd,      // 好友添加
    FriendRecall,   // 私聊消息撤回

    GroupAdmin,     // 群聊管理员变动，需进一步通过SubType确认
    GroupBan,       // 群聊禁言，需进一步通过SubType确认
    GroupIncrease,  // 群聊成员增加，需进一步通过SubType确认
    GroupDecrease,  // 群聊成员减少，需进一步通过SubType确认

    GroupCard,      // 群成员名片更新
    GroupRecall,    // 群聊消息撤回
    GroupUpload,    // 群聊文件上传
    Essence,        // 群聊设精

    Notify,         // 一些通知，需进一步通过SubType确认

    Unknown,        // 未知通知类型
}

impl Serialize for NoticeType {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match self {
            NoticeType::FriendAdd => serializer.serialize_str("friend_add"),
            NoticeType::FriendRecall => serializer.serialize_str("friend_recall"),
            NoticeType::GroupAdmin => serializer.serialize_str("group_admin"),
            NoticeType::GroupBan => serializer.serialize_str("group_ban"),
            NoticeType::GroupIncrease => serializer.serialize_str("group_increase"),
            NoticeType::GroupDecrease => serializer.serialize_str("group_decrease"),
            NoticeType::GroupCard => serializer.serialize_str("group_card"),
            NoticeType::GroupRecall => serializer.serialize_str("group_recall"),
            NoticeType::GroupUpload => serializer.serialize_str("group_upload"),
            NoticeType::Essence => serializer.serialize_str("essence"),
            NoticeType::Notify => serializer.serialize_str("notify"),
            NoticeType::Unknown => serializer.serialize_str("unknown"),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Notice {
    pub notice_type: NoticeType,
    pub sub_type: SubType,
    pub status: String,
    pub user_id: u64,
    pub group_id: Option<u64>,
    pub time: u64,
}

impl Notice{
    pub fn new(data: Value) -> Self {
        let notice_type = match data.get("notice_type").and_then(|v| v.as_str()) {
            Some("friend_add")      => NoticeType::FriendAdd,
            Some("friend_recall")   => NoticeType::FriendRecall,
            Some("group_admin")     => NoticeType::GroupAdmin,
            Some("group_ban")       => NoticeType::GroupBan,
            Some("group_increase")  => NoticeType::GroupIncrease,
            Some("group_decrease")  => NoticeType::GroupDecrease,
            Some("group_card")      => NoticeType::GroupCard,
            Some("group_recall")    => NoticeType::GroupRecall,
            Some("group_upload")    => NoticeType::GroupUpload,
            Some("essence")         => NoticeType::Essence,
            Some("notify")          => NoticeType::Notify,
            _ => NoticeType::Unknown,
        };

        let sub_type = SubType::deserialize(
                data.get("sub_type")
                                    .unwrap_or(&Value::Null))
                                    .unwrap_or(SubType::None);

        let user_id = data.get("user_id").and_then(|v| v.as_u64()).unwrap_or(0);

        let group_id = data.get("group_id").and_then(|v| v.as_u64());

        let time = data.get("time").and_then(|v| v.as_u64()).unwrap_or(0);

        let status = match data.get("event_type").and_then(|v| v.as_i64()) {
            Some(3) => "talking".to_string(),
            Some(1) => "typing".to_string(),
            _ => "unknown".to_string(),
        } ;

        Self { 
            notice_type,
            sub_type,
            status,
            user_id,
            group_id,
            time,
        }
    }
}