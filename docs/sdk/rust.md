# Rust SDK

## 概述

Rust SDK 是 luo9_bot 的官方 SDK，提供完整的功能封装。源码位于 `sdk/rust/`。

## 安装

在插件的 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
luo9_sdk = { path = "../../sdk/rust" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
libc = "0.2"
```

## 模块结构

```
luo9_sdk/
├── lib.rs       # 入口：Bot, Msg, PluginSubscribers, luo9_init_subscribers
├── bus.rs       # Bus 消息总线
├── command.rs   # Command 命令解析
├── pattern.rs   # 模式匹配
├── payload.rs   # Payload 载荷类型
├── message.rs   # MsgBuilder 消息构建器
├── send.rs      # 消息发送（底层）
└── version.rs   # 版本信息
```

## 快速开始

### 基本插件结构

```rust
use luo9_sdk::bus::{Bus, BusError};
use luo9_sdk::payload::BusPayload;
use luo9_sdk::bot::Bot;
use luo9_sdk::command::{Command, PrefixMode};

#[unsafe(no_mangle)]
pub extern "C" fn plugin_main() {
    Bus::init().unwrap();
    let sub_id = Bus::topic("luo9_message").subscribe().unwrap();

    loop {
        match Bus::topic("luo9_message").wait_pop(sub_id) {
            Ok(msg) => {
                if let Some(BusPayload::Message(msg)) = BusPayload::parse(&msg) {
                    handle_message(&msg);
                }
            }
            Err(BusError::Unsubscribed) => break,
            Err(e) => eprintln!("Error: {:?}", e),
        }
    }
}

fn handle_message(msg: &luo9_sdk::payload::MessagePayload) {
    if let Some(cmd) = Command::parse(&msg.message, "echo", PrefixMode::Optional('/')) {
        cmd.handle(|| {
            if let Some(group_id) = msg.group_id {
                Bot::send_group_msg(group_id, "请提供参数".into());
            }
        })
        .on("hello", |_| {
            if let Some(group_id) = msg.group_id {
                Bot::send_group_msg(group_id, "你好！".into());
            }
        })
        .otherwise(|| {
            if let Some(group_id) = msg.group_id {
                Bot::send_group_msg(group_id, format!("你说: {}", cmd.args_raw()).into());
            }
        });
    }
}
```

## API 参考

### Bus 消息总线

```rust
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

### Command 命令解析

```rust
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

### Payload 载荷类型

```rust
BusPayload::parse(json: &str) -> Option<BusPayload>

// 枚举变体
BusPayload::Message(MessagePayload)
BusPayload::MetaEvent(MetaEventPayload)
BusPayload::Notice(NoticePayload)
BusPayload::Request(RequestPayload)
```

详见 [Payload 载荷格式](/sdk/payload)。

### Bot 机器人

```rust
Bot::send_group_msg(group_id: u64, message: CString) -> Option<()>
Bot::send_private_msg(user_id: u64, message: CString) -> Option<()>
Bot::get_version() -> String
```

### 发送模块（底层）

```rust
// 通过 Bus 发送消息（fire-and-forget）
send::send_group_msg(group_id: u64, message: &str) -> Option<()>
send::send_private_msg(user_id: u64, message: &str) -> Option<()>
```

### Msg 消息构建器

`Msg` 提供链式 API 构建 CQ 码消息：

```rust
use luo9_sdk::Msg;

// 构建包含 @、文本、图片的消息
let msg = Msg::txt("你好 ")
    .at(123456)
    .endl()
    .image("https://example.com/image.jpg")
    .build();  // 返回 CString

Bot::send_group_msg(group_id, msg);
```

| 方法 | 说明 |
|---|---|
| `Msg::txt(text)` | 文本消息 |
| `Msg::at(user_id)` | @某人 |
| `Msg::image(path)` | 图片（URL 或本地路径） |
| `Msg::new(text)` | 等同于 `Msg::txt(text)` |
| `.endl()` | 换行 |
| `.build()` | 构建为 `CString` |

## 编译

```bash
# 开发构建
cargo build

# Release 构建
cargo build --release

# 输出文件
# Windows: target/release/plugin_name.dll
# Linux: target/release/libplugin_name.so
```

## 完整示例：定时任务插件

```rust
use luo9_sdk::bus::{Bus, BusError};
use luo9_sdk::bot::Bot;

#[unsafe(no_mangle)]
pub extern "C" fn plugin_main() {
    Bus::init().unwrap();

    // 发布定时任务请求到 luo9_task_miso（在订阅之前）
    let task_req = r#"{
        "action": "schedule",
        "task_name": "my_task",
        "cron": "0 */5 * * * *",
        "payload": "hello"
    }"#;
    Bus::topic("luo9_task_miso").publish(task_req).unwrap();

    // 从 luo9_task 订阅任务事件
    let sub_id = Bus::topic("luo9_task").subscribe().unwrap();

    loop {
        match Bus::topic("luo9_task").wait_pop(sub_id) {
            Ok(msg) => {
                if let Ok(event) = serde_json::from_str::<serde_json::Value>(&msg) {
                    if event["event"] == "tick" {
                        Bot::send_group_msg(789012, "定时任务执行！".into());
                    }
                }
            }
            Err(BusError::Unsubscribed) => break,
            Err(e) => eprintln!("Error: {:?}", e),
        }
    }
}
```

## 注意事项

1. **panic 处理**：`plugin_main` 中的 panic 会被 `catch_unwind` 捕获
2. **内存管理**：`CString` 在传递给 FFI 前需确保以 null 结尾
3. **线程安全**：Bus 函数线程安全，但 subscriber 应在单线程中使用
4. **错误处理**：建议在消息循环中妥善处理错误，避免插件意外退出
