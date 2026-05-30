# Rust SDK API

Rust SDK 的 API 速查。写插件用的。

## 模块结构

```
luo9_sdk/
├── bus.rs       # 消息总线
├── command.rs   # 命令解析
├── pattern.rs   # 模式匹配
├── payload.rs   # 载荷类型
├── message.rs   # 消息构建器
├── send.rs      # 发送（底层）
└── version.rs   # 版本查询
```

## Bus

```rust
use luo9_sdk::bus::{Bus, BusError};

Bus::init() -> Result<(), BusError>
Bus::topic(name: &str) -> Topic

Topic::subscribe() -> Result<usize, BusError>
Topic::unsubscribe(subscriber_id: usize) -> Result<(), BusError>
Topic::publish(payload: &str) -> Result<(), BusError>
Topic::publish_to(payload: &str, subscriber_ids: &[usize]) -> Result<(), BusError>
Topic::pop(subscriber_id: usize) -> Option<String>
Topic::wait_pop(subscriber_id: usize) -> Result<String, BusError>
```

详见 [Bus 消息总线](/sdk/bus)。

## Command

```rust
use luo9_sdk::command::{Command, PrefixMode};

Command::parse(msg: &str, cmd_name: &str, mode: PrefixMode) -> Option<Command>
Command::name() -> &str
Command::args() -> &[String]
Command::arg_at(index: usize) -> Option<&str>
Command::args_raw() -> String
Command::has_args() -> bool
Command::args_count() -> usize
Command::handle(f) -> CommandMatcher
Command::on(expected, f) -> CommandMatcher
Command::on_pattern(pattern, f) -> CommandMatcher
```

详见 [Command 命令解析](/sdk/command)。

## Payload

```rust
use luo9_sdk::payload::BusPayload;

BusPayload::parse(json: &str) -> Option<BusPayload>

BusPayload::Message(MessagePayload)
BusPayload::MetaEvent(MetaEventPayload)
BusPayload::Notice(NoticePayload)
BusPayload::Request(RequestPayload)
```

详见 [Payload 载荷格式](/sdk/payload)。

## Bot

```rust
use luo9_sdk::Bot;

Bot::send_group_msg(group_id: u64, message: CString) -> Option<()>
Bot::send_private_msg(user_id: u64, message: CString) -> Option<()>
Bot::get_version() -> String
```

## Msg 消息构建器

```rust
use luo9_sdk::Msg;

Msg::txt(text)          // 文本
Msg::at(user_id)        // @某人
Msg::image(path)        // 图片（URL 或本地路径）
Msg::new(text)          // 等同于 txt
    .endl()             // 换行
    .build()            // 构建为 CString
```

## version

```rust
use luo9_sdk::version;

version::is_version_query(json: &str) -> bool
version::reply_version(name: &str, version: &str)
```

## send（底层）

```rust
use luo9_sdk::send;

send::send_group_msg(group_id: u64, message: &str) -> Option<()>
send::send_private_msg(user_id: u64, message: &str) -> Option<()>
```

一般用 `Bot` 的方法就够了，不用直接调 send。
