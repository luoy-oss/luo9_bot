# Rust 插件入门

## 开始之前

你需要：
- Rust 工具链（1.75+，推荐 rustup 安装）
- 一个能跑的洛玖机器人（参考 [快速开始](/guide/getting-started)）

## 从模板开始

最快的方式是用模板仓库：

```bash
git clone https://github.com/luo9-bot/luo9_sdk_rust.git my_plugin
cd my_plugin
```

模板已经配好了 `crate-type = ["cdylib"]` 和 `luo9_sdk` 依赖，直接写代码就行。

想从零开始也行：

```bash
cargo new --lib my_plugin
cd my_plugin
```

然后手动配 `Cargo.toml`：

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
    let msg_sub = Bus::topic("luo9_message").subscribe().unwrap();
    let msg_topic = Bus::topic("luo9_message");

    loop {
        if let Some(json) = msg_topic.pop(msg_sub) {
            if let Some(BusPayload::Message(msg)) = BusPayload::parse(&json) {
                eprintln!("收到消息: {}", msg.message);
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
```

编译：

```bash
cargo build --release
```

把 `target/release/my_plugin.dll`（或 `libmy_plugin.so`）丢到宿主的 `plugins/` 目录，重启机器人，就能在日志里看到你的插件在工作了。

## 几个重要的事

### 不要调用 Bus::init()

宿主在加载插件之前已经初始化好了总线。你再调一次会怎样？不会怎样，但没必要。

### 用 pop，别用 wait_pop

`wait_pop` 会阻塞当前线程直到有消息。如果你只订阅一个 topic，用它没问题。但大多数插件需要订阅多个 topic，这时候用 `pop`（非阻塞）+ `sleep` 轮询是更实际的做法。

### 记得处理版本查询

宿主启动时会问每个插件"你是谁"。不回答也没事，但回答了能在 WebUI 里看到插件版本：

```rust
use luo9_sdk::version;

if let Some(json) = ver_topic.pop(ver_sub) {
    if version::is_version_query(&json) {
        version::reply_version(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    }
}
```

## 下一步

- [消息处理](/sdk/rust/messages) — 收发消息、处理通知
- [命令解析](/sdk/rust/commands) — 做一个 `/echo` 机器人
- [定时任务](/sdk/rust/tasks) — 让插件定期干活
