# Payload 载荷格式

## 概述

Payload 模块定义了通过 Bus 传输的消息格式，包括消息、元事件、通知、请求四种类型。

## JSON 格式

所有消息通过 Bus 传输时使用 JSON 字符串，格式为：

```json
{"Message": { ... }}
{"MetaEvent": { ... }}
{"Notice": { ... }}
{"Request": { ... }}
```

## 使用方式

```rust
use luo9_sdk::payload::BusPayload;

let json = r#"{"Message":{"message_type":"group","user_id":123,"group_id":456,"message":"hello"}}"#;

match BusPayload::parse(json) {
    Some(BusPayload::Message(msg)) => {
        println!("消息内容: {}", msg.message);
    }
    Some(BusPayload::Notice(notice)) => {
        println!("通知类型: {:?}", notice.notice_type);
    }
    _ => {}
}
```

## MessagePayload

```rust
pub struct MessagePayload {
    pub message_type: MsgType,      // "private" / "group"
    pub user_id: u64,               // 发送者 QQ 号
    pub group_id: Option<u64>,      // 群号（群消息时存在）
    pub message: String,            // 消息内容
    pub time: u64,                  // 时间戳
    pub self_id: u64,               // 机器人 QQ 号
    pub message_id: u64,            // 消息 ID
    pub message_seq: Option<u64>,   // 消息序列号
    pub real_id: Option<u64>,       // 真实消息 ID
    pub real_seq: Option<String>,   // 真实序列号
    pub sub_type: SubType,          // 子类型
    pub font: u32,                  // 字体
    pub sender: Option<Sender>,     // 发送者信息
    pub anonymous: Option<Anonymous>, // 匿名信息
    pub message_format: String,     // 消息格式
}
```

### MsgType

| 值 | 说明 |
|---|---|
| `Private` | 私聊 |
| `Group` | 群聊 |

### Sender

```rust
pub struct Sender {
    pub user_id: u64,
    pub nickname: String,
    pub card: String,       // 群名片
    pub sex: String,        // 性别
    pub age: u32,
    pub area: String,
    pub level: String,
    pub role: String,       // owner / admin / member
    pub title: String,      // 头衔
}
```

## MetaEventPayload

```rust
pub struct MetaEventPayload {
    pub interval: Option<u64>,
    pub meta_event_type: MetaEventType, // "lifecycle" / "heartbeat"
    pub sub_type: SubType,
    pub self_id: u64,
    pub status: Option<Status>,
    pub time: u64,
}

pub struct Status {
    pub good: bool,
    pub online: bool,
}
```

## NoticePayload

```rust
pub struct NoticePayload {
    pub notice_type: NoticeType,
    pub sub_type: SubType,
    pub user_id: u64,
    pub group_id: Option<u64>,
    pub time: u64,
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
```

### NoticeType

| 值 | 触发场景 | 关键字段 |
|---|---|---|
| `FriendAdd` | 好友添加 | `user_id` |
| `FriendRecall` | 好友消息撤回 | `user_id`, `message_id` |
| `GroupAdmin` | 群管理员变动 | `group_id`, `user_id`, `sub_type` |
| `GroupBan` | 群禁言 | `group_id`, `user_id`, `duration` |
| `GroupIncrease` | 群成员增加 | `group_id`, `user_id`, `operator_id` |
| `GroupDecrease` | 群成员减少 | `group_id`, `user_id`, `operator_id` |
| `GroupCard` | 群名片修改 | `group_id`, `user_id`, `card_new`, `card_old` |
| `GroupRecall` | 群消息撤回 | `group_id`, `user_id`, `message_id` |
| `GroupUpload` | 群文件上传 | `group_id`, `user_id`, `file` |
| `GroupTitle` | 群头衔变更 | `group_id`, `user_id`, `title` |
| `Honor` | 群荣誉变更 | `group_id`, `user_id`, `honor_type` |
| `Essence` | 精华消息 | `group_id`, `user_id`, `message_id` |
| `Poke` | 戳一戳 | `group_id`, `user_id`, `target_id` |
| `LuckyKing` | 运气王 | `group_id`, `user_id`, `target_id` |
| `GroupMsgEmojiLike` | 表情回应 | `group_id`, `user_id`, `message_id` |

## RequestPayload

```rust
pub struct RequestPayload {
    pub request_type: RequestType,      // "friend" / "group"
    pub user_id: u64,
    pub group_id: Option<u64>,
    pub comment: String,
    pub flag: String,
    pub sub_type: GroupRequestSubType,  // "add" / "invite"
    pub time: u64,
    pub self_id: u64,
}
```

## SubType 枚举

| 分类 | 值 | 说明 |
|---|---|---|
| Lifecycle | `enable` / `disable` / `connect` | 生命周期 |
| Message | `friend` / `normal` / `anonymous` | 消息子类型 |
| Notice 管理 | `set` / `unset` | 管理员变动 |
| Notice 禁言 | `ban` / `lift_ban` | 禁言状态 |
| Notice 成员 | `leave` / `kick` / `kick_me` / `approve` / `invite` / `add` | 成员变动 |
| Notice 互动 | `poke` / `lucky_king` | 互动事件 |
| Notice 荣誉 | `talkative` / `performer` / `emotion` / `honor` | 荣誉变更 |

## 示例消息

### 群消息

```json
{
  "Message": {
    "message_type": "group",
    "user_id": 123456,
    "group_id": 789012,
    "message": "你好",
    "raw_message": "你好",
    "time": 1714500000,
    "self_id": 987654,
    "message_id": 12345,
    "sub_type": "normal",
    "sender": {
      "nickname": "测试用户",
      "card": "群名片",
      "role": "member"
    }
  }
}
```

### 心跳事件

```json
{
  "MetaEvent": {
    "meta_event_type": "heartbeat",
    "interval": 5000,
    "status": { "good": true, "online": true },
    "self_id": 987654,
    "time": 1714500000
  }
}
```
