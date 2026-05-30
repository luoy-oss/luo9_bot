# Bus 消息总线

## 它是什么

Bus 是洛玖的核心通信机制。所有消息——不管是 QQ 消息、通知、还是插件发的回复——都通过它传递。

你可以把它想象成一个邮局系统：每个 topic 是一个信箱，每个插件在信箱里有自己的格子。

## Topic

消息按 topic 分类。每个 topic 有独立的队列，互不干扰。

| Topic | 方向 | 用途 |
|---|---|---|
| `luo9_message` | 宿主 → 插件 | QQ 消息 |
| `luo9_meta_event` | 宿主 → 插件 | 心跳、生命周期 |
| `luo9_notice` | 宿主 → 插件 | 好友/群变动 |
| `luo9_request` | 宿主 → 插件 | 好友/群请求 |
| `luo9_task` | 宿主 → 插件 | 定时任务触发 |
| `luo9_task_miso` | 插件 → 宿主 | 创建/取消任务 |
| `luo9_send` | 插件 → 宿主 | 请求发送消息 |
| `luo9_version` | 宿主 → 插件 | 版本查询 |
| `luo9_version_reply` | 插件 → 宿主 | 版本响应 |

## Subscriber

每个插件在每个 topic 上有独立的 subscriber。消息不会被其他插件抢走。

当 subscriber 被取消订阅时（比如插件被禁用），会收到一个特殊的哨兵消息 `__luo9_unsubscribed__`。收到它就该退出循环了。

## 用法

### 订阅

```rust
let sub_id = Bus::topic("luo9_message").subscribe().unwrap();
```

### 取消息

```rust
// 非阻塞，没消息就返回 None
match Bus::topic("luo9_message").pop(sub_id) {
    Some(msg) => { /* 处理消息 */ }
    None => { /* 没消息，干别的去 */ }
}

// 阻塞，等到有消息才返回
match Bus::topic("luo9_message").wait_pop(sub_id) {
    Ok(msg) => { /* 处理消息 */ }
    Err(BusError::Unsubscribed) => { /* 被取消订阅了，退出 */ }
    Err(e) => { /* 出错了 */ }
}
```

多 topic 的插件用 `pop` + `sleep` 轮询；只订阅一个 topic 的可以用 `wait_pop`。

### 发消息

```rust
// 广播给所有订阅者
Bus::topic("luo9_send").publish(r#"{"action":"send_group_msg","group_id":123,"message":"hello"}"#)?;

// 只发给指定的 subscriber
Bus::topic("luo9_message").publish_to("targeted_msg", &[1, 3, 5])?;
```

## 性能

- **无界队列**：消息不会丢，但积压太多会吃内存
- **Per-topic 并发**：不同 topic 互不阻塞
- **阻塞等待**：`wait_pop` 用 Condvar 实现，不占 CPU
