# FFI 接口规范

## 概述

`luo9_core` 是 FFI 接口层，暴露 `extern "C"` 函数供各语言 SDK 调用。

## Bus 消息总线

### luo9_bus_init

```c
int luo9_bus_init();
```

初始化消息总线。

| 返回值 | 含义 |
|---|---|
| `0` | 成功 |
| `-1` | 已初始化（可忽略） |

### luo9_bus_subscribe

```c
int luo9_bus_subscribe(const char* topic);
```

订阅指定 topic。

| 返回值 | 含义 |
|---|---|
| `>= 0` | subscriber_id |
| `-1` | 参数错误 |

### luo9_bus_unsubscribe

```c
int luo9_bus_unsubscribe(const char* topic, int subscriber_id);
```

取消订阅，标记 subscriber 为 dead 并推送哨兵消息。

| 返回值 | 含义 |
|---|---|
| `0` | 成功 |
| `-1` | 参数错误 |
| `-2` | 总线未初始化 |

### luo9_bus_publish

```c
int luo9_bus_publish(const char* topic, const char* payload);
```

广播消息给 topic 的所有 subscriber。

| 返回值 | 含义 |
|---|---|
| `0` | 成功 |
| `-1` | 参数错误 |
| `-2` | 总线未初始化 |

### luo9_bus_publish_to

```c
int luo9_bus_publish_to(
    const char* topic,
    const char* payload,
    const int* subscriber_ids,
    int subscriber_ids_len
);
```

定向推送消息给指定的 subscriber。

### luo9_bus_pop

```c
char* luo9_bus_pop(const char* topic, int subscriber_id);
```

非阻塞取消息。

| 返回值 | 含义 |
|---|---|
| 非 null | 消息字符串（需 `luo9_bus_free_string` 释放） |
| null | 队列为空或错误 |

### luo9_bus_wait_pop

```c
char* luo9_bus_wait_pop(const char* topic, int subscriber_id);
```

阻塞取消息，挂起线程直到有消息。当 subscriber 被取消订阅时返回哨兵消息 `__luo9_unsubscribed__`。

### luo9_bus_free_string

```c
void luo9_bus_free_string(char* ptr);
```

释放由 `luo9_bus_pop` 或 `luo9_bus_wait_pop` 返回的字符串。

## Command 命令解析

### luo9_command_create

```c
CommandHandle* luo9_command_create(
    const char* msg,
    const char* cmd_name,
    int mode,
    char prefix_char
);
```

创建命令解析器。

| mode | 含义 |
|---|---|
| `0` | Required — 必须有前缀 |
| `1` | Optional — 前缀可选 |
| `2` | None — 无前缀 |

| 返回值 | 含义 |
|---|---|
| 非 null | CommandHandle 指针 |
| null | 解析失败 |

### luo9_command_free

```c
void luo9_command_free(CommandHandle* handle);
```

释放命令解析器。

### luo9_command_get_name

```c
char* luo9_command_get_name(const CommandHandle* handle);
```

获取命令名称（需 `luo9_free_string` 释放）。

### luo9_command_get_args_raw

```c
char* luo9_command_get_args_raw(const CommandHandle* handle);
```

获取原始参数字符串（需 `luo9_free_string` 释放）。

### luo9_command_has_args

```c
int luo9_command_has_args(const CommandHandle* handle);
```

| 返回值 | 含义 |
|---|---|
| `1` | 有参数 |
| `0` | 无参数 |
| `-1` | 错误 |

### luo9_command_args_count

```c
int luo9_command_args_count(const CommandHandle* handle);
```

获取参数数量（`-1` 表示错误）。

### luo9_command_get_arg

```c
char* luo9_command_get_arg(const CommandHandle* handle, unsigned int index);
```

获取指定索引的参数（需 `luo9_free_string` 释放）。

### luo9_free_string

```c
void luo9_free_string(char* ptr);
```

释放由 `luo9_command_*` 函数返回的字符串。

## 版本信息

### luo9_version

```c
const char* luo9_version();
```

获取核心库版本号（编译时从 `Cargo.toml` 获取）。返回的字符串由 core 管理，**不需要**释放。

## 插件初始化

### luo9_init_subscribers

```c
void luo9_init_subscribers(const PluginSubscribers* subscribers);
```

传递预分配的 subscriber ID 给插件。

```c
typedef struct {
    int message_sub_id;
    int meta_event_sub_id;
    int notice_sub_id;
    int request_sub_id;
    int task_sub_id;
    int send_sub_id;
} PluginSubscribers;
```

## 内存管理

| 函数 | 释放函数 |
|---|---|
| `luo9_bus_pop` | `luo9_bus_free_string` |
| `luo9_bus_wait_pop` | `luo9_bus_free_string` |
| `luo9_command_get_name` | `luo9_free_string` |
| `luo9_command_get_args_raw` | `luo9_free_string` |
| `luo9_command_get_arg` | `luo9_free_string` |
| `luo9_command_create` | `luo9_command_free` |

## 线程安全

- 所有 `luo9_bus_*` 函数是线程安全的
- 每个 subscriber 应在单线程中使用（pop/wait_pop）
- Command 解析器不是线程安全的，不应跨线程共享
