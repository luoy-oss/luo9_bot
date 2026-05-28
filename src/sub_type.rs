use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Clone)]
pub enum SubType {
    // Lifecycle 子类型
    Enable,
    Disable,
    Connect,

    // Message 子类型
    Friend,       // 好友私聊
    GroupTemp,    // 群临时会话
    GroupSelf,    // 群中自身发送
    Other,        // 其他来源
    Normal,       // 普通消息
    Anonymous,    // 匿名消息
    Notice,       // 系统提示

    // Notice 子类型 - 管理员
    Set,          // 设置管理员
    Unset,        // 取消管理员

    // Notice 子类型 - 禁言
    Ban,          // 禁言
    LiftBan,      // 解禁言
    Unban,        // 解禁言（别名）

    // Notice 子类型 - 成员变动
    Leave,        // 主动退群
    Kick,         // 被踢出群
    KickMe,       // 登录号被踢
    Approve,      // 同意入群/请求
    Invite,       // 邀请入群
    Add,          // 主动加群/添加

    // Notice 子类型 - 戳一戳和互动
    Poke,         // 戳一戳
    LuckyKing,    // 运气王

    // Notice 子类型 - 荣誉
    Talkative,    // 群龙王
    Performer,    // 表演之王
    Emotion,      // 快乐之源

    // Notice 子类型 - 其他
    InputStatus,  // 输入状态
    Title,        // 群头衔变更
    ProfileLike,  // 个人资料点赞
    Honor,        // 群荣誉变更

    None,         // 无子类型/未知
}

impl Serialize for SubType {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match self {
            // Lifecycle
            SubType::Enable => serializer.serialize_str("enable"),
            SubType::Disable => serializer.serialize_str("disable"),
            SubType::Connect => serializer.serialize_str("connect"),

            // Message
            SubType::Friend => serializer.serialize_str("friend"),
            SubType::GroupTemp => serializer.serialize_str("group"),
            SubType::GroupSelf => serializer.serialize_str("group_self"),
            SubType::Other => serializer.serialize_str("other"),
            SubType::Normal => serializer.serialize_str("normal"),
            SubType::Anonymous => serializer.serialize_str("anonymous"),
            SubType::Notice => serializer.serialize_str("notice"),

            // Notice - 管理员
            SubType::Set => serializer.serialize_str("set"),
            SubType::Unset => serializer.serialize_str("unset"),

            // Notice - 禁言
            SubType::Ban => serializer.serialize_str("ban"),
            SubType::LiftBan => serializer.serialize_str("lift_ban"),
            SubType::Unban => serializer.serialize_str("unban"),

            // Notice - 成员变动
            SubType::Leave => serializer.serialize_str("leave"),
            SubType::Kick => serializer.serialize_str("kick"),
            SubType::KickMe => serializer.serialize_str("kick_me"),
            SubType::Approve => serializer.serialize_str("approve"),
            SubType::Invite => serializer.serialize_str("invite"),
            SubType::Add => serializer.serialize_str("add"),

            // Notice - 互动
            SubType::Poke => serializer.serialize_str("poke"),
            SubType::LuckyKing => serializer.serialize_str("lucky_king"),

            // Notice - 荣誉
            SubType::Talkative => serializer.serialize_str("talkative"),
            SubType::Performer => serializer.serialize_str("performer"),
            SubType::Emotion => serializer.serialize_str("emotion"),
            SubType::Honor => serializer.serialize_str("honor"),

            // Notice - 其他
            SubType::InputStatus => serializer.serialize_str("input_status"),
            SubType::Title => serializer.serialize_str("title"),
            SubType::ProfileLike => serializer.serialize_str("profile_like"),

            SubType::None => serializer.serialize_str("none"),
        }
    }
}

impl<'de> Deserialize<'de> for SubType {
    fn deserialize<D>(deserializer: D) -> Result<SubType, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let sub_type = match s.as_str() {
            // Lifecycle
            "enable" => Ok(Self::Enable),
            "disable" => Ok(Self::Disable),
            "connect" => Ok(Self::Connect),

            // Message
            "friend" => Ok(Self::Friend),
            "group" => Ok(Self::GroupTemp),
            "group_self" => Ok(Self::GroupSelf),
            "other" => Ok(Self::Other),
            "normal" => Ok(Self::Normal),
            "anonymous" => Ok(Self::Anonymous),
            "notice" => Ok(Self::Notice),

            // Notice - 管理员
            "set" => Ok(Self::Set),
            "unset" => Ok(Self::Unset),

            // Notice - 禁言
            "ban" => Ok(Self::Ban),
            "lift_ban" => Ok(Self::LiftBan),
            "unban" => Ok(Self::Unban),

            // Notice - 成员变动
            "leave" => Ok(Self::Leave),
            "kick" => Ok(Self::Kick),
            "kick_me" => Ok(Self::KickMe),
            "approve" => Ok(Self::Approve),
            "invite" => Ok(Self::Invite),
            "add" => Ok(Self::Add),

            // Notice - 互动
            "poke" => Ok(Self::Poke),
            "lucky_king" => Ok(Self::LuckyKing),

            // Notice - 荣誉
            "talkative" => Ok(Self::Talkative),
            "performer" => Ok(Self::Performer),
            "emotion" => Ok(Self::Emotion),
            "honor" => Ok(Self::Honor),

            // Notice - 其他
            "input_status" => Ok(Self::InputStatus),
            "title" => Ok(Self::Title),
            "profile_like" => Ok(Self::ProfileLike),

            _ => Ok(Self::None),
        };
        sub_type
    }
}