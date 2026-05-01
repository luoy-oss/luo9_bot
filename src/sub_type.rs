use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Clone)]
pub enum SubType{
  Enable,
  Disable,
  Connect,

  Friend,       // message - 好友
  GroupTemp,    // message - 群临时
  GroupSelf,    // message - 群中自身发送
  Other,        // message - 其他
  Normal,       // message - 普通
  Notice,       // message - 系统提示

  Set,          // notice  - 增加
  Unset,        // notice  - 减少
  Ban,          // notice  - 禁言
  LiftBan,      // notice  - 解禁言
  Leave,        // notice  - 退出群聊
  Kick,         // notice  - 踢出群聊
  KickMe,       // notice  - 登录号被踢
  Approve,      // notice  - 同意入群

  Poke,         // 戳一戳

  InputStatus,   // notice  - 输入状态
  Title,        // notice  - 群头衔变更
  ProfileLike,  // notice  - 个人资料点赞
 
  Add,          //添加
  Invite,       //邀请

  None,      // 无子类型
}

impl Serialize for SubType {
  fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
    match self {
      SubType::Enable => serializer.serialize_str("enable"),
      SubType::Disable => serializer.serialize_str("disable"),
      SubType::Connect => serializer.serialize_str("connect"),
      SubType::Friend => serializer.serialize_str("friend"),
      SubType::GroupTemp => serializer.serialize_str("group"),
      SubType::GroupSelf => serializer.serialize_str("group_self"),
      SubType::Other => serializer.serialize_str("other"),
      SubType::Normal => serializer.serialize_str("normal"),
      SubType::Notice => serializer.serialize_str("notice"),
      SubType::Set => serializer.serialize_str("set"),
      SubType::Unset => serializer.serialize_str("unset"),
      SubType::Ban => serializer.serialize_str("ban"),
      SubType::LiftBan => serializer.serialize_str("lift_ban"),
      SubType::Leave => serializer.serialize_str("leave"),
      SubType::Kick => serializer.serialize_str("kick"),
      SubType::KickMe => serializer.serialize_str("kick_me"),
      SubType::Approve => serializer.serialize_str("approve"),
      SubType::Poke => serializer.serialize_str("poke"),
      SubType::InputStatus => serializer.serialize_str("input_status"),
      SubType::Title => serializer.serialize_str("title"),
      SubType::ProfileLike => serializer.serialize_str("profile_like"),
      SubType::Add => serializer.serialize_str("add"),
      SubType::Invite => serializer.serialize_str("invite"),
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
      "enable"        => Ok(Self::Enable),
      "disable"       => Ok(Self::Disable),
      "connect"       => Ok(Self::Connect),
      "friend"        => Ok(Self::Friend),
      "group"         => Ok(Self::GroupTemp),
      "group_self"    => Ok(Self::GroupSelf),
      "other"         => Ok(Self::Other),
      "normal"        => Ok(Self::Normal),
      "notice"        => Ok(Self::Notice),
      "set"           => Ok(Self::Set),
      "unset"         => Ok(Self::Unset),
      "ban"           => Ok(Self::Ban),
      "lift_ban"       => Ok(Self::LiftBan),
      "leave"         => Ok(Self::Leave),
      "kick"          => Ok(Self::Kick),
      "kick_me"       => Ok(Self::KickMe),
      "approve"       => Ok(Self::Approve),
      "poke"          => Ok(Self::Poke),
      "input_status"   => Ok(Self::InputStatus),
      "title"         => Ok(Self::Title),
      "profile_like"  => Ok(Self::ProfileLike),
      "add"           => Ok(Self::Add),
      "invite"        => Ok(Self::Invite),
      _               => Ok(Self::None),
    };
    sub_type
  }
}