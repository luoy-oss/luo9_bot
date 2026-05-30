# Python 插件开发

::: warning 即将推出
Python SDK 正在开发中，敬请期待。
:::

## 当前状态

Python SDK 的核心功能已经实现，但文档和示例还在完善中。

如果你等不及，可以参考：

- [FFI 接口规范](/sdk/ffi-interface) — Python 通过 ctypes 调用这些 C 函数
- [Rust 插件开发](/sdk/rust/) — 参考 Rust SDK 的实现思路

## 快速了解

Python 插件的基本结构：

```python
from luo9_sdk import Bus, Bot, BusPayload

def plugin_main():
    Bus.init()
    sub_id = Bus.topic("luo9_message").subscribe()

    while True:
        try:
            msg = Bus.topic("luo9_message").wait_pop(sub_id)
            payload = BusPayload.parse(msg)
            if payload and payload.message_type == "group":
                Bot.send_group_msg(payload.group_id, "收到！")
        except RuntimeError:
            break  # 收到取消订阅信号
```

详细文档正在编写中。
