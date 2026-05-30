# FFI 接口规范

`luo9_core` 暴露的 C 函数。SDK 开发者看这篇。

## Bus 消息总线

### luo9_bus_init

```c
int luo9_bus_init();
```

初始化总线。返回 `0` 成功，`-1` 已经初始化过（可忽略）。

### luo9_bus_subscribe

```c
int luo9_bus_subscribe(const char* topic);
```

订阅 topic。返回 subscriber_id（>=0），失败返回 `-1`。

### luo9_bus_unsubscribe

```c
int luo9_bus_unsubscribe(const char* topic, int subscriber_id);
```

取消订阅。会推送哨兵消息唤醒阻塞的 `wait_pop`。

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

### luo9_bus_publish_to

```c
int luo9_bus_publish_to(const char* topic, const char* payload, const int* ids, int ids_len);
```

定向推送给指定的 subscriber。

### luo9_bus_pop

```c
char* luo9_bus_pop(const char* topic, int subscriber_id);
```

非阻塞取消息。有消息返回字符串指针，没有返回 `null`。

### luo9_bus_wait_pop

```c
char* luo9_bus_wait_pop(const char* topic, int subscriber_id);
```

阻塞取消息。挂起线程直到有消息。取消订阅时返回哨兵 `__luo9_unsubscribed__`。

### luo9_bus_free_string

```c
void luo9_bus_free_string(char* ptr);
```

释放 `luo9_bus_pop` 和 `luo9_bus_wait_pop` 返回的字符串。

## Command 命令解析

### luo9_command_create

```c
CommandHandle* luo9_command_create(const char* msg, const char* cmd_name, int mode, char prefix_char);
```

创建命令解析器。

| mode | 含义 |
|---|---|
| `0` | 必须有前缀 |
| `1` | 前缀可选 |
| `2` | 无前缀 |

返回句柄指针，解析失败返回 `null`。

### luo9_command_free

```c
void luo9_command_free(CommandHandle* handle);
```

释放句柄。

### luo9_command_get_name

```c
char* luo9_command_get_name(const CommandHandle* handle);
```

获取命令名。返回的字符串用 `luo9_free_string` 释放。

### luo9_command_get_args_raw

```c
char* luo9_command_get_args_raw(const CommandHandle* handle);
```

获取原始参数字符串。

### luo9_command_has_args

```c
int luo9_command_has_args(const CommandHandle* handle);
```

`1` 有参数，`0` 无参数，`-1` 错误。

### luo9_command_args_count

```c
int luo9_command_args_count(const CommandHandle* handle);
```

参数数量。`-1` 表示错误。

### luo9_command_get_arg

```c
char* luo9_command_get_arg(const CommandHandle* handle, unsigned int index);
```

获取第 N 个参数。越界返回 `null`。

### luo9_free_string

```c
void luo9_free_string(char* ptr);
```

释放 `luo9_command_*` 返回的字符串。

## 版本

### luo9_version

```c
const char* luo9_version();
```

获取核心库版本。返回的字符串由 core 管理，**不需要**释放。

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

## 内存管理速查

| 谁返回的 | 用什么释放 |
|---|---|
| `luo9_bus_pop` / `luo9_bus_wait_pop` | `luo9_bus_free_string` |
| `luo9_command_get_name` 等 | `luo9_free_string` |
| `luo9_command_create` | `luo9_command_free` |
| `luo9_version` | 不用释放 |

## 线程安全

- `luo9_bus_*` 函数线程安全
- 每个 subscriber 建议在单线程中使用
- Command 句柄不要跨线程共享
