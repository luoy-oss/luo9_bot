# 命令解析

## 基本用法

用 `Command::parse` 从消息里提取命令：

```rust
use luo9_sdk::command::{Command, PrefixMode};

if let Some(cmd) = Command::parse(&msg.message, "echo", PrefixMode::Required('/')) {
    let text = cmd.args().join(" ");
    Bot::send_group_msg(group_id, CString::new(text).unwrap());
}
```

消息是 `/echo hello world` 时：
- `cmd.name()` → `"echo"`
- `cmd.args()` → `["hello", "world"]`
- `cmd.args_raw()` → `" hello world"`（注意前面有空格）

消息不匹配时返回 `None`。

## 前缀模式

| 模式 | 行为 | 例子 |
|---|---|---|
| `Required('/')` | 必须有 `/` | `/echo hello` ✓，`echo hello` ✗ |
| `Optional('/')` | 可选 | `/echo hello` ✓，`echo hello` ✓ |
| `None` | 不需要前缀 | `echo hello` ✓ |

## 链式匹配多个子命令

```rust
if let Some(cmd) = Command::parse(&msg.message, "task", PrefixMode::Required('/')) {
    cmd.on("start", |args| {
        // /task start ...
    })
    .on("stop", |args| {
        // /task stop ...
    })
    .on("list", |_| {
        // /task list
    })
    .otherwise(|| {
        // /task（没跟参数或不认识的子命令）
    });
}
```

`on` 的第一个参数是子命令名，匹配时执行闭包。`otherwise` 在所有 `on` 都没匹配时执行。

## 模式匹配

想从消息里提取结构化数据？用 `on_pattern`：

```rust
cmd.on_pattern("[CQ:at,qq={qq}]{content}", |caps, args| {
    let qq = caps.get("qq").unwrap();
    let content = caps.get("content").unwrap();
    println!("被 @ 的人: {}, 内容: {}", qq, content);
});
```

`{name}` 是命名捕获，会把对应位置的文本提取出来。

## 无参数命令

```rust
cmd.handle(|| {
    // /help（没有参数时执行）
})
.on("verbose", |_| {
    // /help verbose
});
```

## 完整示例

```rust
use luo9_sdk::Bot;
use luo9_sdk::command::{Command, PrefixMode};
use std::ffi::CString;

fn handle_group_msg(group_id: u64, msg: &str) {
    // /echo <text>
    if let Some(cmd) = Command::parse(msg, "echo", PrefixMode::Required('/')) {
        Bot::send_group_msg(group_id, CString::new(cmd.args().join(" ")).unwrap());
        return;
    }

    // /help
    if let Some(cmd) = Command::parse(msg, "help", PrefixMode::Required('/')) {
        cmd.handle(|| {
            Bot::send_group_msg(group_id, CString::new("可用命令: /echo, /help").unwrap());
        });
    }
}
```
