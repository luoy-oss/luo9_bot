# Payload 载荷格式

## 它是什么

从总线取消息时，拿到的是一段 JSON 字符串。Payload 模块帮你把它解析成结构化的 Rust 类型。

## JSON 格式

消息的 JSON 最外层有一个类型标识：

```json
// QQ 消息
{"Message": {"message_type": "group", "user_id": 123, "message": "hello", ...}}

// 元事件
{"MetaEvent": {"meta_event_type": "heartbeat", ...}}

// 通知
{"Notice": {"notice_type": "group_increase", ...}}

// 请求
{"Request": {"request_type": "friend", ...}}
```

## 解析

```rust
use luo9_sdk::payload::BusPayload;

let json = r#"{"Message":{"message_type":"group","user_id":123,"group_id":456,"message":"hello"}}"#;

match BusPayload::parse(json) {
    Some(BusPayload::Message(msg)) => {
        println!("消息: {}", msg.message);
    }
    Some(BusPayload::Notice(notice)) => {
        println!("通知: {:?}", notice.notice_type);
    }
    _ => {}
}
```

## MessagePayload

QQ 消息的结构。

| 字段 | 类型 | 说明 |
|---|---|---|
| `message_type` | `MsgType` | `Private` / `Group` |
| `user_id` | `u64` | 发送者 QQ 号 |
| `group_id` | `Option<u64>` | 群号（群消息时有） |
| `message` | `String` | 消息文本 |
| `time` | `u64` | 时间戳 |
| `self_id` | `u64` | 机器人 QQ 号 |
| `message_id` | `u64` | 消息 ID |
| `sub_type` | `SubType` | 子类型 |
| `sender` | `Option<Sender>` | 发送者信息 |

## MetaEventPayload

心跳和生命周期事件。

| 字段 | 说明 |
|---|---|
| `meta_event_type` | `Lifecycle` / `Heartbeat` |
| `interval` | 心跳间隔（毫秒） |
| `status` | 状态信息（good, online） |

## NoticePayload

好友/群变动通知。

| NoticeType | 触发场景 | 关键字段 |
|---|---|---|
| `FriendAdd` | 好友添加 | `user_id` |
| `FriendRecall` | 好友撤回 | `user_id`, `message_id` |
| `GroupAdmin` | 管理员变动 | `sub_type`(Set/Unset) |
| `GroupBan` | 禁言 | `duration` |
| `GroupIncrease` | 成员增加 | `operator_id` |
| `GroupDecrease` | 成员减少 | `operator_id` |
| `GroupCard` | 名片修改 | `card_new`, `card_old` |
| `GroupRecall` | 群消息撤回 | `operator_id`, `message_id` |
| `GroupUpload` | 文件上传 | `file` |
| `Poke` | 戳一戳 | `target_id` |

## RequestPayload

好友/群请求。

| RequestType | 字段 |
|---|---|
| `Friend` | `user_id`, `comment`, `flag` |
| `Group` | `group_id`, `user_id`, `comment`, `flag`, `sub_type` |
