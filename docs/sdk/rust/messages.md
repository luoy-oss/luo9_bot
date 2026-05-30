# 消息处理

## 多 Topic 轮询

实际插件通常要同时监听多个 topic。用 `pop` + `sleep` 是最实用的方式：

```rust
#[unsafe(no_mangle)]
pub extern "C" fn plugin_main() {
    let msg_sub = Bus::topic("luo9_message").subscribe().unwrap();
    let event_sub = Bus::topic("luo9_meta_event").subscribe().unwrap();
    let notice_sub = Bus::topic("luo9_notice").subscribe().unwrap();
    let ver_sub = Bus::topic("luo9_version").subscribe().unwrap();

    let msg_topic = Bus::topic("luo9_message");
    let event_topic = Bus::topic("luo9_meta_event");
    let notice_topic = Bus::topic("luo9_notice");
    let ver_topic = Bus::topic("luo9_version");

    loop {
        // 消息
        if let Some(json) = msg_topic.pop(msg_sub) {
            if let Some(BusPayload::Message(msg)) = BusPayload::parse(&json) {
                // 处理消息...
            }
        }

        // 元事件
        if let Some(json) = event_topic.pop(event_sub) {
            if let Some(BusPayload::MetaEvent(ev)) = BusPayload::parse(&json) {
                // 处理元事件...
            }
        }

        // 通知
        if let Some(json) = notice_topic.pop(notice_sub) {
            if let Some(BusPayload::Notice(notice)) = BusPayload::parse(&json) {
                // 处理通知...
            }
        }

        // 版本查询
        if let Some(json) = ver_topic.pop(ver_sub) {
            if luo9_sdk::version::is_version_query(&json) {
                luo9_sdk::version::reply_version(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
```

## 区分消息类型

`msg.message_type` 告诉你这是群消息还是私聊：

```rust
match msg.message_type {
    MsgType::Group => {
        let group_id = msg.group_id.unwrap_or(0);
        // 群消息处理
    }
    MsgType::Private => {
        let user_id = msg.user_id;
        // 私聊处理
    }
    _ => {}
}
```

## 发送消息

```rust
use luo9_sdk::Bot;
use std::ffi::CString;

// 回复群消息
Bot::send_group_msg(group_id, CString::new("你好！").unwrap());

// 回复私聊
Bot::send_private_msg(user_id, CString::new("你好！").unwrap());
```

## 处理通知

通知事件告诉你发生了什么：

```rust
use luo9_sdk::payload::*;

fn handle_notice(notice: NoticePayload) {
    match notice.notice_type {
        NoticeType::GroupIncrease => {
            // 新成员入群，可以发个欢迎消息
        }
        NoticeType::FriendAdd => {
            // 新好友
        }
        NoticeType::Poke => {
            // 被戳了
        }
        _ => {}
    }
}
```

## 处理元事件

```rust
fn handle_meta_event(ev: MetaEventPayload) {
    match ev.meta_event_type {
        MetaEventType::Heartbeat => {
            // 心跳，可以用来检测在线状态
        }
        MetaEventType::Lifecycle => {
            // 生命周期事件
        }
        _ => {}
    }
}
```

## 消息构建器

想发 @某人 + 图片这种复杂消息？用 `Msg`：

```rust
use luo9_sdk::Msg;

let msg = Msg::txt("看看这个 ")
    .at(123456)           // @某人
    .endl()               // 换行
    .image("https://example.com/pic.jpg")
    .build();             // 构建为 CString

Bot::send_group_msg(group_id, msg);
```

| 方法 | 说明 |
|---|---|
| `Msg::txt(text)` | 文本 |
| `Msg::at(user_id)` | @某人 |
| `Msg::image(path)` | 图片（URL 或本地路径） |
| `.endl()` | 换行 |
| `.build()` | 构建为 CString |
