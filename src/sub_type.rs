use serde::Deserialize;

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