# Command 命令解析

## 概述

Command 模块提供命令解析功能，用于从消息中提取命令和参数。

## 核心概念

### 前缀模式

| 模式 | 说明 | 示例 |
|---|---|---|
| `Required(char)` | 必须有前缀 | `/echo hello` |
| `Optional(char)` | 前缀可选 | `echo hello` 或 `/echo hello` |
| `None` | 无前缀 | `echo hello` |

### 命令结构

```
/prefix command arg1 arg2 ...
   ↑       ↑      ↑
 前缀    命令名  参数
```

## 使用方式

### 基本用法

```rust
use luo9_sdk::command::{Command, PrefixMode};

let msg = "/echo hello world";
let cmd = Command::parse(msg, "echo", PrefixMode::Required('/'));

if let Some(cmd) = cmd {
    println!("命令名: {}", cmd.name());        // "echo"
    println!("参数数: {}", cmd.args_count());   // 2
    println!("参数0: {}", cmd.arg_at(0).unwrap()); // "hello"
    println!("参数1: {}", cmd.arg_at(1).unwrap()); // "world"
    println!("原始参数: {}", cmd.args_raw());   // " hello world"
}
```

### 链式匹配

```rust
let cmd = Command::parse("/task start my_task", "task", PrefixMode::Required('/')).unwrap();

cmd.on("start", |args| {
    println!("启动任务: {:?}", args);
})
.on("stop", |args| {
    println!("停止任务: {:?}", args);
})
.on("list", |_| {
    println!("列出任务");
})
.otherwise(|| {
    println!("未知子命令");
});
```

### 模式匹配

```rust
let msg = "[CQ:at,qq=123456]状态";
let cmd = Command::parse(msg, "epic", PrefixMode::None).unwrap();

cmd.on_pattern("[CQ:at,qq={qq}]{content}", |caps, args| {
    let qq = caps.get("qq").unwrap();
    let content = caps.get("content").unwrap();
    println!("@{}: {}", qq, content);
});
```

### 无参数命令

```rust
let cmd = Command::parse("/help", "help", PrefixMode::Required('/')).unwrap();

cmd.handle(|| {
    println!("显示帮助信息");
})
.on("verbose", |_| {
    println!("显示详细帮助");
});
```

## API 参考

### Command::parse

```rust
pub fn parse(msg: &str, cmd_name: &str, mode: PrefixMode) -> Option<Self>
```

解析消息，匹配返回 `Some(Command)`，否则 `None`。

### Command 方法

| 方法 | 返回值 | 说明 |
|---|---|---|
| `name()` | `&str` | 命令名称 |
| `args()` | `&[String]` | 所有参数 |
| `arg_at(index)` | `Option<&str>` | 指定索引的参数 |
| `args_from(start)` | `&[String]` | 从指定索引开始的参数 |
| `args_raw()` | `String` | 原始参数字符串 |
| `has_args()` | `bool` | 是否有参数 |
| `args_count()` | `usize` | 参数数量 |
| `handle(f)` | `CommandMatcher` | 无参数时执行闭包 |
| `on(expected, f)` | `CommandMatcher` | 第一个参数匹配时执行 |
| `on_pattern(pattern, f)` | `CommandMatcher` | 模式匹配 |

### CommandMatcher 方法

| 方法 | 说明 |
|---|---|
| `on(expected, f)` | 链式匹配 |
| `on_pattern(pattern, f)` | 链式模式匹配 |
| `otherwise(f)` | 所有匹配失败时执行 |

## 模式语法

| 语法 | 说明 | 示例 |
|---|---|---|
| `{name}` | 命名捕获 | `{qq}` 捕获为 "qq" |
| `[CQ:at,qq={qq}]` | 捕获 CQ 码参数 | `[CQ:at,qq=123456]` |
| `{content}` | 捕获剩余内容 | 任意文本 |

## 最佳实践

1. **命令唯一性**：确保命令名在插件内唯一
2. **参数验证**：检查参数数量和类型
3. **错误提示**：参数不足时给出使用提示
4. **链式匹配**：使用 `on`/`otherwise` 处理多个子命令
