# 开发新 SDK

本指南面向希望为 luo9_bot 开发新语言 SDK 的开发者。

## 概述

SDK 是对 `luo9_core` FFI 接口的语言封装层。开发新 SDK 的核心工作是：

1. 链接 `luo9_core.dll` / `libluo9_core.so`
2. 声明 FFI 函数
3. 封装为该语言的惯用 API

## 步骤 1：链接 luo9_core

SDK 需要链接 `luo9_core.dll`（Windows）或 `libluo9_core.so`（Linux）。

## 步骤 2：声明 FFI 函数

需要声明以下 `extern "C"` 函数（详见 [FFI 接口规范](/sdk/ffi-interface)）：

```c
// Bus 消息总线
int luo9_bus_init();
int luo9_bus_subscribe(const char* topic);
int luo9_bus_unsubscribe(const char* topic, int subscriber_id);
int luo9_bus_publish(const char* topic, const char* payload);
int luo9_bus_publish_to(const char* topic, const char* payload, const int* ids, int ids_len);
char* luo9_bus_pop(const char* topic, int subscriber_id);
char* luo9_bus_wait_pop(const char* topic, int subscriber_id);
void luo9_bus_free_string(char* ptr);

// 版本
const char* luo9_version();

// Command 命令解析
CommandHandle* luo9_command_create(const char* msg, const char* cmd_name, int mode, char prefix);
void luo9_command_free(CommandHandle* handle);
char* luo9_command_get_name(const CommandHandle* handle);
char* luo9_command_get_args_raw(const CommandHandle* handle);
int luo9_command_has_args(const CommandHandle* handle);
int luo9_command_args_count(const CommandHandle* handle);
char* luo9_command_get_arg(const CommandHandle* handle, unsigned int index);
void luo9_free_string(char* ptr);
```

## 步骤 3：实现 Bus 封装

核心逻辑：

```rust
pub struct Bus;
impl Bus {
    pub fn init() -> Result<(), BusError> { /* luo9_bus_init */ }
    pub fn topic(name: &str) -> Topic { Topic { name } }
}

pub struct Topic<'a> { name: &'a str }
impl<'a> Topic<'a> {
    pub fn subscribe(&self) -> Result<usize, BusError> {
        // 1. 检查预分配 ID（PRECREATED_SUBSCRIBERS）
        // 2. 若无，调用 luo9_bus_subscribe
    }
    pub fn pop(&self, subscriber_id: usize) -> Option<String> {
        // 调用 luo9_bus_pop
        // 若返回 sentinel "__luo9_unsubscribed__"，返回 None
    }
    pub fn wait_pop(&self, subscriber_id: usize) -> Result<String, BusError> {
        // 调用 luo9_bus_wait_pop
        // 若返回 sentinel "__luo9_unsubscribed__"，返回 Err(BusError::Unsubscribed)
    }
    pub fn publish(&self, payload: &str) -> Result<(), BusError> { /* luo9_bus_publish */ }
    pub fn publish_to(&self, payload: &str, ids: &[usize]) -> Result<(), BusError> { /* luo9_bus_publish_to */ }
}
```

## 步骤 4：实现 Command 封装

```rust
pub struct Command { handle: *mut CommandHandle }
impl Command {
    pub fn parse(msg: &str, cmd_name: &str, mode: PrefixMode) -> Option<Self> {
        // 调用 luo9_command_create
    }
    pub fn name(&self) -> &str { /* luo9_command_get_name */ }
    pub fn args_raw(&self) -> String { /* luo9_command_get_args_raw */ }
    pub fn has_args(&self) -> bool { /* luo9_command_has_args */ }
    pub fn args_count(&self) -> usize { /* luo9_command_args_count */ }
    pub fn arg_at(&self, index: usize) -> Option<&str> { /* luo9_command_get_arg */ }
}
```

## 步骤 5：实现 Payload 解析

```rust
#[derive(Debug, Deserialize)]
pub enum BusPayload {
    Message(MessagePayload),
    MetaEvent(MetaEventPayload),
    Notice(NoticePayload),
    Request(RequestPayload),
}
impl BusPayload {
    pub fn parse(json: &str) -> Option<Self> { serde_json::from_str(json).ok() }
}
```

## 步骤 6：实现 Bot 封装

```rust
pub struct Bot;
impl Bot {
    pub fn send_group_msg(group_id: u64, message: &str) -> Option<()> {
        // 构造 JSON → Bus::topic("luo9_send").publish(json)
    }
    pub fn send_private_msg(user_id: u64, message: &str) -> Option<()> { /* 同理 */ }
}
```

## 步骤 7：实现版本查询

宿主启动时会查询插件版本，SDK 需要提供响应机制：

```rust
pub fn is_version_query(json: &str) -> bool {
    // 检查 action == "query"
}

pub fn reply_version(name: &str, version: &str) {
    // 发送到 luo9_version_reply topic
}
```

## 参考实现

- [Rust SDK](/sdk/rust) — 完整参考实现（`sdk/rust/`）
- [FFI 接口规范](/sdk/ffi-interface) — 所有 FFI 函数的详细说明

## 注意事项

1. **字符串内存管理**：`luo9_bus_pop`、`luo9_bus_wait_pop`、`luo9_command_*` 返回的字符串必须用对应的 free 函数释放
2. **线程安全**：Bus 函数是线程安全的，但每个 subscriber 应在单线程中使用
3. **错误处理**：所有函数都有错误返回值，应妥善处理
4. **取消订阅**：收到 sentinel 消息 `__luo9_unsubscribed__` 时应退出循环
