# 技巧与常见问题

## 常见问题

### 插件不加载

- 检查 `Cargo.toml` 有没有 `crate-type = ["cdylib"]`
- 检查有没有导出 `plugin_main`，有没有加 `#[unsafe(no_mangle)]`
- 看宿主日志，通常会告诉你哪里出了问题

### 收不到消息

- 确认订阅了正确的 topic
- 确认用的是 `pop` 而不是 `wait_pop`（多 topic 场景）
- 确认循环里有 `sleep`，不然线程会疯狂空转

### 编译报错找不到 luo9_sdk

- 确认 `Cargo.toml` 里写了 `luo9_sdk = "0.7.1"`
- 跑一下 `cargo update`

## 最佳实践

### 错误处理

单个消息处理失败不应该让插件退出：

```rust
loop {
    if let Some(json) = msg_topic.pop(msg_sub) {
        if let Some(BusPayload::Message(msg)) = BusPayload::parse(&json) {
            // 用 unwrap_or 处理可能的错误
            let group_id = msg.group_id.unwrap_or(0);
            if let Err(e) = process_message(group_id, &msg.message) {
                eprintln!("处理消息出错: {}", e);
            }
        }
    }
    std::thread::sleep(std::time::Duration::from_millis(1));
}
```

### 日志输出

用 `eprintln!` 输出调试信息，宿主会捕获并显示在日志里：

```rust
eprintln!("[my_plugin] 收到消息: {}", msg.message);
```

### 线程安全

Bus 函数是线程安全的，但每个 subscriber 建议在单线程中使用。如果你需要多线程，每个线程应该有自己的 subscriber。

### 资源清理

`plugin_main` 返回后，线程就结束了。如果有什么需要清理的资源，在返回前处理好。

## 进阶

- [Bus 消息总线](/sdk/bus) — 底层通信机制
- [Command 命令解析](/sdk/command) — 更多命令处理技巧
- [Payload 载荷格式](/sdk/payload) — 完整的消息结构
