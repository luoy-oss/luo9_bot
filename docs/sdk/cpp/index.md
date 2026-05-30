# C++ 插件开发

::: warning 即将推出
C++ SDK 正在开发中，敬请期待。
:::

## 当前状态

C++ SDK 的核心功能已经实现，但文档和示例还在完善中。

如果你等不及，可以参考：

- [FFI 接口规范](/sdk/ffi-interface) — C++ 直接调用这些 C 函数即可
- [Rust 插件开发](/sdk/rust/) — 参考 Rust SDK 的实现思路

## 快速了解

C++ 插件的基本结构：

```cpp
#include "luo9_sdk.h"

extern "C" void plugin_main() {
    Bus::init();
    int sub_id = Bus::topic("luo9_message").subscribe();

    while (true) {
        try {
            std::string msg = Bus::topic("luo9_message").wait_pop(sub_id);
            // 处理消息...
        } catch (const std::runtime_error&) {
            break;  // 收到取消订阅信号
        }
    }
}
```

详细文档正在编写中。
