# Bus 消息总线

## 概述

Bus 是 luo9_bot 插件通信的核心，提供发布/订阅模式的消息传递。

## 核心概念

### Topic（主题）

消息按主题分类，每个主题有独立的消息队列：

| Topic | 方向 | 用途 |
|---|---|---|
| `luo9_message` | Host → Plugin | QQ 消息 |
| `luo9_meta_event` | Host → Plugin | 元事件 |
| `luo9_notice` | Host → Plugin | 通知事件 |
| `luo9_request` | Host → Plugin | 请求事件 |
| `luo9_task` | Host → Plugin | 定时任务事件（tick 下发） |
| `luo9_task_miso` | Plugin → Host | 定时任务请求（schedule/cancel） |
| `luo9_send` | Plugin → Host | 消息发送 |
| `luo9_version` | Host → Plugin | 版本查询请求 |
| `luo9_version_reply` | Plugin → Host | 版本查询响应 |

### Subscriber（订阅者）

每个插件在每个 topic 上有独立的 subscriber，拥有独立的消息队列。

### Sentinel（哨兵）

当 subscriber 被取消订阅时，会收到哨兵消息 `__luo9_unsubscribed__`，表示应退出循环。

## 使用方式

### 初始化

```rust
use luo9_sdk::bus::{Bus, BusError};

Bus::init()?;
```

### 订阅 Topic

```rust
let sub_id = Bus::topic("luo9_message").subscribe()?;
```

### 接收消息

```rust
// 阻塞取消息（推荐）
match Bus::topic("luo9_message").wait_pop(sub_id) {
    Ok(msg) => println!("收到消息: {}", msg),
    Err(BusError::Unsubscribed) => {
        // 被取消订阅，应退出循环
        break;
    }
    Err(e) => eprintln!("错误: {:?}", e),
}

// 非阻塞取消息
match Bus::topic("luo9_message").pop(sub_id) {
    Some(msg) => println!("收到消息: {}", msg),
    None => println!("队列为空"),
}
```

### 发布消息

```rust
// 广播消息
Bus::topic("luo9_send").publish(r#"{"action":"send_group_msg","group_id":123,"message":"hello"}"#)?;

// 定向推送
let target_ids = vec![1, 3, 5];
Bus::topic("luo9_message").publish_to("targeted_msg", &target_ids)?;
```

## 典型插件结构

```rust
use luo9_sdk::bus::{Bus, BusError};
use luo9_sdk::payload::BusPayload;
use luo9_sdk::bot::Bot;

#[unsafe(no_mangle)]
pub extern "C" fn plugin_main() {
    Bus::init().unwrap();
    let sub_id = Bus::topic("luo9_message").subscribe().unwrap();

    loop {
        match Bus::topic("luo9_message").wait_pop(sub_id) {
            Ok(msg) => {
                if let Some(BusPayload::Message(msg)) = BusPayload::parse(&msg) {
                    if msg.message == "你好" {
                        Bot::send_group_msg(msg.group_id.unwrap(), "你好！".into());
                    }
                }
            }
            Err(BusError::Unsubscribed) => break,
            Err(e) => eprintln!("Error: {:?}", e),
        }
    }
}
```

## 消息格式

所有消息通过 Bus 传输时使用 JSON 字符串（详见 [Payload 载荷格式](/sdk/payload)）。

## BusError 枚举

```rust
pub enum BusError {
    InitFailed,        // 初始化失败
    PublishFailed,     // 发布失败
    SubscribeFailed,   // 订阅失败
    WaitPopFailed,     // 阻塞取消息失败
    NotInitialized,    // 总线未初始化
    InvalidString,     // 字符串错误
    UnsubscribeFailed, // 取消订阅失败
    Unsubscribed,      // 已取消订阅（收到哨兵）
}
```

## 性能特性

- **无界队列**：不丢消息，但可能消耗大量内存
- **Per-topic 并发**：不同 topic 的 publish/pop 互不阻塞
- **阻塞等待**：`wait_pop` 使用 Condvar，不消耗 CPU
