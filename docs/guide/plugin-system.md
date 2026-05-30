# 插件系统

::: tip
想要编写插件？请查看 [Rust 插件开发指南](/sdk/rust-plugin-dev)。
:::

## 概述

luo9_bot 的插件系统基于 FFI 消息总线设计，插件是独立的共享库（DLL/SO），通过 `luo9_core` 提供的 `extern "C"` 函数进行进程内 pub/sub 通信。

## 插件生命周期

### 1. 加载阶段

```
宿主扫描插件目录
    ↓
为每个插件预创建 subscriber（各 topic）
    ↓
通过 luo9_init_subscribers FFI 传递 ID
    ↓
插件的 SDK 内部透明使用预分配 ID
```

### 2. 运行阶段

```
插件在独立 OS 线程中运行
    ↓
通过 Bus::topic().subscribe() 获取 subscriber_id
    ↓
循环 wait_pop() 接收消息
    ↓
处理消息并通过 Bot::send_*() 回复
```

### 3. 禁用 / 热重载

```
调用 unsubscribe_all() 标记 dead + 唤醒线程
    ↓
插件 wait_pop 收到 sentinel __luo9_unsubscribed__
    ↓
循环退出 → 线程结束 → Arc<Library> 释放 → dlclose
    ↓
（热重载时）重新加载 .dll/.so，创建新 subscriber，spawn 新线程
```

**插件代码零修改**即可支持热重载。

## 插件入口函数

插件必须导出 `plugin_main` 函数，在独立线程中运行：

```rust
#[unsafe(no_mangle)]
pub extern "C" fn plugin_main() {
    // 初始化总线
    Bus::init().unwrap();

    // 订阅消息
    let sub_id = Bus::topic("luo9_message").subscribe().unwrap();

    // 消息循环
    loop {
        match Bus::topic("luo9_message").wait_pop(sub_id) {
            Ok(msg) => {
                // 处理消息
            }
            Err(BusError::Unsubscribed) => {
                // 收到取消订阅信号，退出循环
                break;
            }
            Err(e) => {
                eprintln!("Error: {:?}", e);
            }
        }
    }
}
```

## 消息处理流程

```
Napcat 推送消息
    ↓
Receiver 接收 WebSocket 消息
    ↓
handler/core.rs 解析 PostType
    ↓
handler/message.rs / event.rs / notice.rs
    ↓
plugin::dispatch_*() 按优先级分发
    ↓
Bus::publish_to() 定向推送给指定 subscriber
    ↓
插件 wait_pop() 收到消息
    ↓
插件处理并通过 Bus::topic("luo9_send").publish() 回复
    ↓
plugin/sender.rs 接收发送请求 → Sender 发送到 Napcat API
```

## 消息格式

所有消息通过 Bus 传输时使用统一的 JSON 格式（详见 [Payload 载荷格式](/sdk/payload)）：

```json
{
  "Message": {
    "message_type": "group",
    "user_id": 123456,
    "group_id": 789012,
    "message": "你好",
    "time": 1714500000,
    "self_id": 987654,
    "message_id": 12345,
    "sender": {
      "nickname": "测试用户",
      "role": "member"
    }
  }
}
```

## 发送消息

插件通过向 `luo9_send` topic 发送 JSON 请求来发送消息：

```json
{"action": "send_group_msg", "group_id": 789012, "message": "收到你的消息了！"}
```

```json
{"action": "send_private_msg", "user_id": 123456, "message": "这是私聊回复"}
```

## 定时任务

插件可以向 `luo9_task_miso` topic 发布任务请求：

```json
{"action": "schedule", "task_name": "my_task", "cron": "0 */5 * * * *", "payload": "任意数据"}
```

取消任务：

```json
{"action": "cancel", "task_name": "my_task"}
```

宿主调度器到期后向 `luo9_task` topic 发布事件：

```json
{"event": "tick", "task_name": "my_task", "payload": "任意数据"}
```

**注意**：任务请求发到 `luo9_task_miso`，任务事件从 `luo9_task` 接收。插件应在订阅 `luo9_task` **之前**发布 task 请求到 `luo9_task_miso`，否则会通过 latch 机制收到自己的请求。

## 最佳实践

1. **独立线程**：插件运行在独立 OS 线程，阻塞调用不会影响 tokio 运行时
2. **错误恢复**：单个消息处理失败不应导致插件退出
3. **资源清理**：在 `plugin_main` 返回前清理所有资源
4. **日志输出**：使用 `eprintln!` 输出调试信息，宿主会捕获并显示
5. **优先级设置**：合理设置插件优先级，避免消息处理顺序混乱
