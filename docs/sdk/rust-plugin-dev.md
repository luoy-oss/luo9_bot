# Rust 插件开发指南

## 开始之前

你需要：
- Rust 工具链（1.75+，推荐 rustup 安装）
- 一个能跑的洛玖机器人（参考 [快速开始](/guide/getting-started)）

准备好了？继续往下看。

## 从模板开始

最快的方式是用模板仓库：

```bash
git clone https://github.com/luo9-bot/luo9_sdk_rust.git my_plugin
cd my_plugin
```

模板已经帮你配好了 `Cargo.toml` 和 `crate-type = ["cdylib"]`，直接写代码就行。

如果你想从零开始，也可以：

```bash
cargo new --lib my_plugin
cd my_plugin
```

然后手动配置 `Cargo.toml`：

```toml
[package]
name = "my_plugin"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]   # 必须，不然编译不出 DLL/SO

[dependencies]
luo9_sdk = "0.7.1"
serde_json = "1.0"
```

## 第一个插件

打开 `src/lib.rs`，写入：

```rust
use luo9_sdk::bus::Bus;
use luo9_sdk::payload::*;

#[unsafe(no_mangle)]
pub extern "C" fn plugin_main() {
    // 订阅消息
    let msg_sub = Bus::topic("luo9_message").subscribe().unwrap();
    let msg_topic = Bus::topic("luo9_message");

    loop {
        // 有消息就处理
        if let Some(json) = msg_topic.pop(msg_sub) {
            if let Some(BusPayload::Message(msg)) = BusPayload::parse(&json) {
                eprintln!("收到消息: {}", msg.message);
            }
        }
        // 别忘了让出 CPU，不然会空转
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
```

编译：

```bash
cargo build --release
```

把生成的 `target/release/my_plugin.dll`（或 `libmy_plugin.so`）丢到宿主的 `plugins/` 目录，重启机器人，就能在日志里看到你的插件在工作了。

## 几个重要的事

### 不要调用 Bus::init()

宿主在加载插件之前已经初始化好了总线。你再调一次会怎样？不会怎样，但没必要。

### 用 pop，别用 wait_pop

`wait_pop` 会阻塞当前线程直到有消息。如果你只订阅一个 topic，用它没问题。但大多数插件需要订阅多个 topic，这时候用 `pop`（非阻塞）+ `sleep` 轮询是更实际的做法。

### 记得处理版本查询

宿主启动时会问每个插件"你是谁"。不回答也没事，但回答了能在 WebUI 里看到插件版本：

```rust
use luo9_sdk::version;

// 在循环里加一段
if let Some(json) = ver_topic.pop(ver_sub) {
    if version::is_version_query(&json) {
        version::reply_version(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    }
}
```

## 发送消息

收到消息后，你通常想回复点什么：

```rust
use luo9_sdk::Bot;
use std::ffi::CString;

// 回复群消息
Bot::send_group_msg(group_id, CString::new("你好！").unwrap());

// 回复私聊
Bot::send_private_msg(user_id, CString::new("你好！").unwrap());
```

## 命令解析

做机器人少不了命令处理。SDK 提供了 `Command` 来帮你解析：

```rust
use luo9_sdk::command::{Command, PrefixMode};

// 解析 "/echo hello world"
if let Some(cmd) = Command::parse(&msg.message, "echo", PrefixMode::Required('/')) {
    let text = cmd.args().join(" ");
    Bot::send_group_msg(group_id, CString::new(text).unwrap());
}
```

多个子命令？用链式匹配：

```rust
if let Some(cmd) = Command::parse(&msg.message, "task", PrefixMode::Required('/')) {
    cmd.on("start", |args| {
        // /task start ...
    })
    .on("stop", |args| {
        // /task stop ...
    })
    .otherwise(|| {
        // /task （啥也没跟）
    });
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

## 定时任务

想让插件定期做点事情？

```rust
use luo9_sdk::bus::Bus;
use serde_json::json;

// 创建任务（发到 luo9_task_miso）
let req = json!({
    "action": "schedule",
    "task_name": "daily_report",
    "cron": "0 0 9 * * *",    // 每天早上 9 点
    "payload": "日报时间到"
});
Bus::topic("luo9_task_miso").publish(&req.to_string()).unwrap();
```

任务触发后，你会从 `luo9_task` 收到事件：

```json
{"event": "tick", "task_name": "daily_report", "payload": "日报时间到"}
```

取消任务：

```rust
let cancel = json!({ "action": "cancel", "task_name": "daily_report" });
Bus::topic("luo9_task_miso").publish(&cancel.to_string()).unwrap();
```

## 完整示例：Echo 插件

一个完整的插件大概长这样：

```rust
use luo9_sdk::Bot;
use luo9_sdk::bus::Bus;
use luo9_sdk::command::{Command, PrefixMode};
use luo9_sdk::payload::*;
use std::ffi::CString;

fn handle_group_msg(group_id: u64, msg: &str) {
    if let Some(cmd) = Command::parse(msg, "echo", PrefixMode::Required('/')) {
        Bot::send_group_msg(group_id, CString::new(cmd.args().join(" ")).unwrap());
    }
}

fn handle_private_msg(user_id: u64, msg: &str) {
    if let Some(cmd) = Command::parse(msg, "echo", PrefixMode::Required('/')) {
        Bot::send_private_msg(user_id, CString::new(cmd.args().join(" ")).unwrap());
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn plugin_main() {
    let msg_sub = Bus::topic("luo9_message").subscribe().unwrap();
    let ver_sub = Bus::topic("luo9_version").subscribe().unwrap();
    let msg_topic = Bus::topic("luo9_message");
    let ver_topic = Bus::topic("luo9_version");

    loop {
        if let Some(json) = msg_topic.pop(msg_sub) {
            if let Some(BusPayload::Message(msg)) = BusPayload::parse(&json) {
                match msg.message_type {
                    MsgType::Group => handle_group_msg(msg.group_id.unwrap_or(0), &msg.message),
                    MsgType::Private => handle_private_msg(msg.user_id, &msg.message),
                    _ => {}
                }
            }
        }

        if let Some(json) = ver_topic.pop(ver_sub) {
            if luo9_sdk::version::is_version_query(&json) {
                luo9_sdk::version::reply_version(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
```

## 常见问题

**插件不加载？**
- 检查 `Cargo.toml` 有没有 `crate-type = ["cdylib"]`
- 检查有没有导出 `plugin_main`，有没有加 `#[unsafe(no_mangle)]`
- 看宿主日志，通常会告诉你哪里出了问题

**收不到消息？**
- 确认订阅了正确的 topic
- 确认用的是 `pop` 而不是 `wait_pop`（多 topic 场景）
- 确认循环里有 `sleep`，不然线程会疯狂空转

**编译报错找不到 luo9_sdk？**
- 确认 `Cargo.toml` 里写了 `luo9_sdk = "0.7.1"`
- 跑一下 `cargo update`
