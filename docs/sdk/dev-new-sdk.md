# 开发新 SDK

想给其他语言写 SDK？这篇说清楚需要对接什么。

## 核心思路

SDK 的工作很简单：把 `luo9_core` 的 C 函数包装成该语言的惯用 API。

```
C 函数（luo9_core）
    ↓ 你的 SDK
语言惯用 API（比如 Python 的 Bus.topic("luo9_message").pop()）
    ↓ 插件开发者用
插件
```

你需要做的事情：
1. 链接 `luo9_core.dll` / `libluo9_core.so`
2. 声明 FFI 函数
3. 包装成该语言的风格

## 要对接哪些函数

完整的函数列表在 [FFI 接口规范](/sdk/ffi-interface)，这里列一下核心的几个：

**消息总线**（必须）：
- `luo9_bus_init()` — 初始化总线
- `luo9_bus_subscribe(topic)` — 订阅 topic
- `luo9_bus_unsubscribe(topic, sub_id)` — 取消订阅
- `luo9_bus_publish(topic, payload)` — 发消息
- `luo9_bus_publish_to(topic, payload, ids, len)` — 定向发消息
- `luo9_bus_pop(topic, sub_id)` — 非阻塞取消息
- `luo9_bus_wait_pop(topic, sub_id)` — 阻塞取消息
- `luo9_bus_free_string(ptr)` — 释放字符串

**命令解析**（推荐）：
- `luo9_command_create(...)` — 创建命令解析器
- `luo9_command_get_name(handle)` — 获取命令名
- `luo9_command_get_args_raw(handle)` — 获取原始参数
- `luo9_command_has_args(handle)` — 有没有参数
- `luo9_command_args_count(handle)` — 参数数量
- `luo9_command_get_arg(handle, index)` — 获取第 N 个参数
- `luo9_command_free(handle)` — 释放解析器
- `luo9_free_string(ptr)` — 释放字符串

**版本**（推荐）：
- `luo9_version()` — 获取核心库版本

## 关键细节

### 预分配 Subscriber

宿主在加载插件时会预创建 subscriber，然后通过 `luo9_init_subscribers` 传给插件。SDK 的 `subscribe()` 应该先检查有没有预分配的 ID，有的话直接用，不用再调 FFI。

```c
typedef struct {
    int message_sub_id;
    int meta_event_sub_id;
    int notice_sub_id;
    int request_sub_id;
    int task_sub_id;
    int send_sub_id;
} PluginSubscribers;

void luo9_init_subscribers(const PluginSubscribers* subscribers);
```

### 哨兵消息

取消订阅时，`wait_pop` 会返回一个特殊字符串 `__luo9_unsubscribed__`。SDK 应该识别它，转成该语言的错误类型（比如异常、Error 枚举等）。

### 字符串释放

`luo9_bus_pop`、`luo9_bus_wait_pop`、`luo9_command_*` 返回的字符串是 C 分配的，必须用对应的 free 函数释放。忘了释放就是内存泄漏。

### 插件入口

插件必须导出 `plugin_main` 函数。宿主在独立线程里调用它。

## 参考实现

- [Rust SDK](/sdk/rust) — 完整参考，看看 Rust 是怎么包装的
- [FFI 接口规范](/sdk/ffi-interface) — 所有函数的签名和返回值
