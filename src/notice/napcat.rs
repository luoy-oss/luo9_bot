use crate::sub_type::SubType;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;

/// 群文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub name: String,
    pub size: u64,
    pub busid: i64,
}

/// 群荣誉类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HonorType {
    #[serde(rename = "talkative")]
    Talkative, // 群龙王
    #[serde(rename = "performer")]
    Performer, // 表演之王
    #[serde(rename = "emotion")]
    Emotion, // 快乐之源
}

/// OneBot v11 通知事件类型
#[derive(Debug, Clone)]
pub enum NoticeType {
    // 好友相关
    FriendAdd,    // 好友添加
    FriendRecall, // 私聊消息撤回
    FriendPoke,   // 好友戳一戳（通过 notify）

    // 群成员变动
    GroupAdmin,    // 群聊管理员变动
    GroupBan,      // 群聊禁言
    GroupIncrease, // 群聊成员增加
    GroupDecrease, // 群聊成员减少

    // 群消息相关
    GroupCard,   // 群成员名片更新
    GroupRecall, // 群聊消息撤回
    GroupUpload, // 群聊文件上传

    // 群荣誉和头衔
    GroupTitle, // 群头衔变更
    Honor,      // 群荣誉变更

    // 精华消息
    Essence, // 群聊设精

    // 戳一戳和互动
    Poke,      // 戳一戳（群内）
    LuckyKing, // 运气王

    // 表情回应（NapCat 扩展）
    GroupMsgEmojiLike, // 群消息表情回应

    // 通用通知
    Notify, // 其他通知，需进一步通过SubType确认

    Unknown, // 未知通知类型
}

impl Serialize for NoticeType {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match self {
            NoticeType::FriendAdd => serializer.serialize_str("friend_add"),
            NoticeType::FriendRecall => serializer.serialize_str("friend_recall"),
            NoticeType::FriendPoke => serializer.serialize_str("friend_poke"),
            NoticeType::GroupAdmin => serializer.serialize_str("group_admin"),
            NoticeType::GroupBan => serializer.serialize_str("group_ban"),
            NoticeType::GroupIncrease => serializer.serialize_str("group_increase"),
            NoticeType::GroupDecrease => serializer.serialize_str("group_decrease"),
            NoticeType::GroupCard => serializer.serialize_str("group_card"),
            NoticeType::GroupRecall => serializer.serialize_str("group_recall"),
            NoticeType::GroupUpload => serializer.serialize_str("group_upload"),
            NoticeType::GroupTitle => serializer.serialize_str("group_title"),
            NoticeType::Honor => serializer.serialize_str("honor"),
            NoticeType::Essence => serializer.serialize_str("essence"),
            NoticeType::Poke => serializer.serialize_str("poke"),
            NoticeType::LuckyKing => serializer.serialize_str("lucky_king"),
            NoticeType::GroupMsgEmojiLike => serializer.serialize_str("group_msg_emoji_like"),
            NoticeType::Notify => serializer.serialize_str("notify"),
            NoticeType::Unknown => serializer.serialize_str("unknown"),
        }
    }
}

/// 完整的 OneBot v11 通知事件结构
#[derive(Debug, Serialize, Clone)]
pub struct Notice {
    pub time: u64,
    pub self_id: u64,
    pub post_type: String,
    pub notice_type: NoticeType,
    pub sub_type: SubType,
    pub user_id: u64,
    pub group_id: Option<u64>,
    pub operator_id: Option<u64>,
    pub target_id: Option<u64>,
    pub message_id: Option<u64>,
    pub file: Option<FileInfo>,
    pub duration: Option<u64>,
    pub card_new: Option<String>,
    pub card_old: Option<String>,
    pub honor_type: Option<HonorType>,
    pub title: Option<String>,
    pub flag: Option<String>,
    pub comment: Option<String>,
}

impl Notice {
    pub fn new(data: Value) -> Self {
        let notice_type = match data.get("notice_type").and_then(|v| v.as_str()) {
            // 好友相关
            Some("friend_add") => NoticeType::FriendAdd,
            Some("friend_recall") => NoticeType::FriendRecall,

            // 群管理相关
            Some("group_admin") => NoticeType::GroupAdmin,
            Some("group_ban") => NoticeType::GroupBan,
            Some("group_increase") => NoticeType::GroupIncrease,
            Some("group_decrease") => NoticeType::GroupDecrease,

            // 群消息相关
            Some("group_card") => NoticeType::GroupCard,
            Some("group_recall") => NoticeType::GroupRecall,
            Some("group_upload") => NoticeType::GroupUpload,

            // 精华消息
            Some("essence") => NoticeType::Essence,

            // 通知类（包含戳一戳、运气王等）
            Some("notify") => match data.get("sub_type").and_then(|v| v.as_str()) {
                Some("poke") => NoticeType::Poke,
                Some("lucky_king") => NoticeType::LuckyKing,
                Some("honor") => NoticeType::Honor,
                Some("title") => NoticeType::GroupTitle,
                _ => NoticeType::Notify,
            },

            // NapCat 扩展
            Some("group_msg_emoji_like") => NoticeType::GroupMsgEmojiLike,

            _ => NoticeType::Unknown,
        };

        let sub_type = SubType::deserialize(data.get("sub_type").unwrap_or(&Value::Null))
            .unwrap_or(SubType::None);

        let user_id = data.get("user_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let group_id = data.get("group_id").and_then(|v| v.as_u64());
        let operator_id = data.get("operator_id").and_then(|v| v.as_u64());
        let target_id = data.get("target_id").and_then(|v| v.as_u64());
        let message_id = data.get("message_id").and_then(|v| v.as_u64());

        let file = data
            .get("file")
            .and_then(|v| serde_json::from_value(v.clone()).ok());
        let duration = data.get("duration").and_then(|v| v.as_u64());

        let card_new = data
            .get("card_new")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let card_old = data
            .get("card_old")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let honor_type = data
            .get("honor_type")
            .and_then(|v| serde_json::from_value(v.clone()).ok());

        let title = data
            .get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let flag = data
            .get("flag")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let comment = data
            .get("comment")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let time = data.get("time").and_then(|v| v.as_u64()).unwrap_or(0);
        let self_id = data.get("self_id").and_then(|v| v.as_u64()).unwrap_or(0);

        Self {
            time,
            self_id,
            post_type: "notice".to_string(),
            notice_type,
            sub_type,
            user_id,
            group_id,
            operator_id,
            target_id,
            message_id,
            file,
            duration,
            card_new,
            card_old,
            honor_type,
            title,
            flag,
            comment,
        }
    }
}
