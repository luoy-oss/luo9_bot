# Command 命令解析

## 它能做什么

帮你从消息里提取命令和参数。比如 `/echo hello world`，它能拆出命令名 `echo` 和参数 `["hello", "world"]`。

## 前缀模式

| 模式 | 行为 | 例子 |
|---|---|---|
| `Required('/')` | 必须有前缀 | `/echo hello` ✓，`echo hello` ✗ |
| `Optional('/')` | 前缀可选 | `/echo hello` ✓，`echo hello` ✓ |
| `None` | 不需要前缀 | `echo hello` ✓ |

## 基本用法

```rust
use luo9_sdk::command::{Command, PrefixMode};

let cmd = Command::parse("/echo hello world", "echo", PrefixMode::Required('/')).unwrap();

cmd.name();        // "echo"
cmd.args();        // ["hello", "world"]
cmd.args_count();  // 2
cmd.arg_at(0);     // Some("hello")
cmd.args_raw();    // " hello world"（注意前面有空格）
```

`parse` 返回 `None` 表示消息不匹配这个命令。

## 链式匹配

处理多个子命令时，用 `on` 链式调用：

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

## 模式匹配

支持从消息里提取结构化数据，比如 CQ 码：

```rust
cmd.on_pattern("[CQ:at,qq={qq}]{content}", |caps, args| {
    let qq = caps.get("qq").unwrap();       // "123456"
    let content = caps.get("content").unwrap(); // 剩余文本
});
```

`{name}` 是命名捕获，会把对应位置的文本提取出来。

## 无参数命令

如果命令本身就没有参数，用 `handle`：

```rust
cmd.handle(|| {
    // /help（没有参数时执行）
})
.on("verbose", |_| {
    // /help verbose
});
```
