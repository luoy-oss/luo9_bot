# Rust 插件开发指南

本指南面向**插件开发者**，说明如何用 Rust 编写 luo9_bot 插件。

## 快速开始

### 方式一：使用模板仓库（推荐）

```bash
git clone https://github.com/luo9-bot/luo9_sdk_rust.git my_plugin
cd my_plugin
# 修改 Cargo.toml 中的 name 和 description
```

模板仓库已配置好 `crate-type = ["cdylib"]` 和 `luo9_sdk` 依赖，可以直接开始编写插件代码。

### 方式二：从零创建

```bash
cargo new --lib my_plugin
cd my_plugin
```

### 2. 配置 Cargo.toml

```toml
[package]
name = "my_plugin"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]   # 必须：生成 DLL/SO 共享库

[dependencies]
luo9_sdk = "0.7.1"         # 从 crates.io 安装
serde_json = "1.0"
```

**关键**：`crate-type = ["cdylib"]` 必须设置，否则不会生成 `.dll` / `.so` 文件。

### 3. 编写插件

```rust
// src/lib.rs
use luo9_sdk::bus::Bus;
use luo9_sdk::payload::*;

#[unsafe(no_mangle)]
pub extern "C" fn plugin_main() {
    // 订阅需要的 topic
    let msg_sub = Bus::topic("luo9_message").subscribe().unwrap();
    let msg_topic = Bus::topic("luo9_message");

    loop {
        // 非阻塞取消息
        if let Some(json) = msg_topic.pop(msg_sub) {
            if let Some(BusPayload::Message(msg)) = BusPayload::parse(&json) {
                // 处理消息
                eprintln!("收到消息: {}", msg.message);
            }
        }
        // 短暂让出 CPU，避免空转
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
```

### 4. 编译

```bash
cargo build --release
```

输出文件：
- Windows: `target/release/my_plugin.dll`
- Linux: `target/release/libmy_plugin.so`

### 5. 部署

将编译产物复制到宿主的 `plugins/` 目录。

## 插件入口

插件必须导出 `plugin_main` 函数：

```rust
#[unsafe(no_mangle)]
pub extern "C" fn plugin_main() {
    // 你的代码
}
```

- 宿主在**独立 OS 线程**中调用此函数
- panic 会被 `catch_unwind` 捕获，不会导致宿主崩溃
- 函数返回后线程结束

## 重要：不要调用 Bus::init()

宿主在加载插件之前已经初始化了总线。插件**不需要**也**不应该**调用 `Bus::init()`。

## 多 Topic 处理模式

实际插件通常需要订阅多个 topic。推荐使用 `pop`（非阻塞）+ `sleep` 轮询：

```rust
#[unsafe(no_mangle)]
pub extern "C" fn plugin_main() {
    // 订阅所有需要的 topic
    let msg_sub = Bus::topic("luo9_message").subscribe().unwrap();
    let event_sub = Bus::topic("luo9_meta_event").subscribe().unwrap();
    let notice_sub = Bus::topic("luo9_notice").subscribe().unwrap();
    let task_sub = Bus::topic("luo9_task").subscribe().unwrap();
    let ver_sub = Bus::topic("luo9_version").subscribe().unwrap();

    let msg_topic = Bus::topic("luo9_message");
    let event_topic = Bus::topic("luo9_meta_event");
    let notice_topic = Bus::topic("luo9_notice");
    let task_topic = Bus::topic("luo9_task");
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

        // 定时任务事件
        if let Some(json) = task_topic.pop(task_sub) {
            // 处理任务事件...
        }

        // 版本查询
        if let Some(json) = ver_topic.pop(ver_sub) {
            if luo9_sdk::version::is_version_query(&json) {
                luo9_sdk::version::reply_version(
                    env!("CARGO_PKG_NAME"),
                    env!("CARGO_PKG_VERSION"),
                );
            }
        }

        // 短暂让出 CPU，避免空转
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
```

## 版本查询

宿主启动时会向 `luo9_version` topic 发送查询，插件需要响应：

```rust
// 订阅版本查询 topic
let ver_sub = Bus::topic("luo9_version").subscribe().unwrap();
let ver_topic = Bus::topic("luo9_version");

// 在循环中处理
if let Some(json) = ver_topic.pop(ver_sub) {
    if luo9_sdk::version::is_version_query(&json) {
        luo9_sdk::version::reply_version(
            env!("CARGO_PKG_NAME"),    // 插件名
            env!("CARGO_PKG_VERSION"), // 插件版本
        );
    }
}
```

## 发送消息

```rust
use luo9_sdk::Bot;
use std::ffi::CString;

// 发送群消息
Bot::send_group_msg(group_id, CString::new("你好！").unwrap());

// 发送私聊消息
Bot::send_private_msg(user_id, CString::new("你好！").unwrap());
```

## 命令解析

```rust
use luo9_sdk::command::{Command, PrefixMode};

// 解析 /echo hello world
if let Some(cmd) = Command::parse(&msg.message, "echo", PrefixMode::Required('/')) {
    let text = cmd.args().join(" ");
    Bot::send_group_msg(group_id, CString::new(text).unwrap());
}

// 链式匹配多个子命令
if let Some(cmd) = Command::parse(&msg.message, "task", PrefixMode::Required('/')) {
    cmd.on("start", |args| {
        // /task start ...
    })
    .on("stop", |args| {
        // /task stop ...
    })
    .otherwise(|| {
        // /task （无参数或其他）
    });
}
```

## 消息构建器

使用 `Msg` 链式 API 构建包含 CQ 码的消息：

```rust
use luo9_sdk::Msg;

// 构建 @某人 + 文本 + 图片
let msg = Msg::txt("你好 ")
    .at(123456)
    .endl()
    .image("https://example.com/image.jpg")
    .build();

Bot::send_group_msg(group_id, msg);
```

## 定时任务

```rust
use luo9_sdk::bus::Bus;
use serde_json::json;

// 发布定时任务请求（发到 luo9_task_miso）
let req = json!({
    "action": "schedule",
    "task_name": "my_task",
    "cron": "0 */5 * * * *",   // 每 5 分钟
    "payload": "hello"
});
Bus::topic("luo9_task_miso").publish(&req.to_string()).unwrap();

// 取消任务
let cancel = json!({
    "action": "cancel",
    "task_name": "my_task"
});
Bus::topic("luo9_task_miso").publish(&cancel.to_string()).unwrap();

// 接收任务事件（从 luo9_task）
// 事件格式: {"event":"tick","task_name":"my_task","payload":"hello"}
```

**注意**：任务请求发到 `luo9_task_miso`，任务事件从 `luo9_task` 接收。

## Topic 一览

| Topic | 方向 | 用途 |
|---|---|---|
| `luo9_message` | Host → Plugin | QQ 消息 |
| `luo9_meta_event` | Host → Plugin | 元事件（心跳/生命周期） |
| `luo9_notice` | Host → Plugin | 通知事件 |
| `luo9_request` | Host → Plugin | 请求事件 |
| `luo9_task` | Host → Plugin | 定时任务事件（tick） |
| `luo9_task_miso` | Plugin → Host | 定时任务请求（schedule/cancel） |
| `luo9_send` | Plugin → Host | 消息发送（SDK 内部使用） |
| `luo9_version` | Host → Plugin | 版本查询 |
| `luo9_version_reply` | Plugin → Host | 版本响应（SDK 内部使用） |

## 完整示例：Echo 插件

```rust
// src/lib.rs
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
                    MsgType::Group => {
                        handle_group_msg(msg.group_id.unwrap_or(0), &msg.message);
                    }
                    MsgType::Private => {
                        handle_private_msg(msg.user_id, &msg.message);
                    }
                    _ => {}
                }
            }
        }

        if let Some(json) = ver_topic.pop(ver_sub) {
            if luo9_sdk::version::is_version_query(&json) {
                luo9_sdk::version::reply_version(
                    env!("CARGO_PKG_NAME"),
                    env!("CARGO_PKG_VERSION"),
                );
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
```

## 常见问题

### 插件不加载

1. 确认 `crate-type = ["cdylib"]` 已设置
2. 确认导出了 `plugin_main` 函数
3. 确认使用了 `#[unsafe(no_mangle)]`
4. 检查日志中的加载错误信息

### 插件收不到消息

1. 确认订阅了正确的 topic
2. 确认使用 `pop` 而非 `wait_pop`（多 topic 场景）
3. 确认调用了 `std::thread::sleep` 避免空转

### 编译错误：找不到 luo9_sdk

确认 `Cargo.toml` 中的依赖版本正确：
```toml
luo9_sdk = "0.7.1"
```
