# 事件类型

luo9_bot 全量适配 Napcat OneBot v11 协议。

## 消息事件

```json
{
  "PostType": "message",
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
    "sender": { "nickname": "测试用户", "role": "member" }
  }
}
```

### 子类型

| 值 | 说明 |
|---|---|
| `friend` | 好友消息 |
| `normal` | 普通群消息 |
| `anonymous` | 匿名消息 |
| `group_self` | 自己发送 |

## 通知事件

### NoticeType

| 类型 | 触发场景 | 关键字段 |
|---|---|---|
| `friend_add` | 好友添加 | `user_id` |
| `friend_recall` | 好友撤回 | `user_id`, `message_id` |
| `group_admin` | 管理员变动 | `sub_type`(Set/Unset) |
| `group_ban` | 群禁言 | `duration`, `sub_type`(Ban/LiftBan) |
| `group_increase` | 成员增加 | `sub_type`(Approve/Invite) |
| `group_decrease` | 成员减少 | `sub_type`(Leave/Kick/KickMe) |
| `group_card` | 名片修改 | `card_new`, `card_old` |
| `group_recall` | 群消息撤回 | `operator_id`, `message_id` |
| `group_upload` | 文件上传 | `file` |
| `group_title` | 头衔变更 | `title` |
| `honor` | 荣誉变更 | `honor_type` |
| `essence` | 精华消息 | `message_id` |
| `poke` | 戳一戳 | `target_id` |
| `lucky_king` | 运气王 | `target_id` |
| `group_msg_emoji_like` | 表情回应 | `message_id` |

## 请求事件

| 类型 | 字段 | 说明 |
|---|---|---|
| `friend` | `user_id`, `comment`, `flag` | 好友请求 |
| `group` | `group_id`, `user_id`, `comment`, `flag`, `sub_type` | 群请求 |

## 元事件

| 类型 | 字段 | 说明 |
|---|---|---|
| `lifecycle` | `sub_type`(Enable/Disable/Connect) | 生命周期 |
| `heartbeat` | `status`, `interval` | 心跳 |

## 消息段

| 类型 | 说明 |
|---|---|
| `text` | 文本 |
| `face` | QQ 表情 |
| `image` | 图片 |
| `at` | @某人 |
| `reply` | 回复 |
| `json` | JSON 消息 |
| `record` | 语音 |
| `video` | 视频 |

## CQ 码

| CQ 码 | 说明 |
|---|---|
| `[CQ:face,id=14]` | QQ 表情 |
| `[CQ:image,file=abc.image]` | 图片 |
| `[CQ:at,qq=123456]` | @某人 |
| `[CQ:reply,id=12345]` | 回复 |
| `[CQ:record,file=abc.record]` | 语音 |
