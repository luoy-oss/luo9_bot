# 插件类型与配置指南

luo9_bot 支持四种插件运行时，插件文件放入配置的 `plugin_dir` 目录即可被自动发现和加载。

## 目录结构

```
plugins/
├── my_native.dll          # 原生插件（Windows）
├── libmy_native.so        # 原生插件（Linux）
├── my_python.py           # Python 插件
├── my_java.jar            # Java 插件
├── my_js.js               # JavaScript 插件（QuickJS）
└── my_java/               # Java 插件（含依赖目录）
    ├── my_java.jar
    └── lib/
        └── gson-2.10.jar  # 依赖 JAR 自动加入 classpath
```

## 1. 原生插件（DLL/SO）

**文件格式**：`.dll`（Windows）/ `.so`（Linux）

**编译要求**：动态链接库，导出 `plugin_main` 符号

**Feature flag**：无需额外 feature，始终可用

**示例**（Rust 插件）：
```rust
// plugin/src/lib.rs
use luo9_sdk::bus::Bus;

#[no_mangle]
pub extern "C" fn plugin_main() {
    let sub = Bus::topic("luo9_message").subscribe();
    loop {
        if let Some(json) = Bus::topic("luo9_message").pop(sub) {
            // 处理消息...
        }
    }
}
```

**编译命令**：
```bash
# Rust
cargo build --release

# C++
g++ -shared -o my_plugin.dll plugin.cpp -L/path/to/luo9_core.dll
```

**SDK 依赖**：
- Rust: `luo9_sdk` crate
- C++: `sdk/cpp/` 头文件 + `luo9_core.dll`

---

## 2. Python 插件

**文件格式**：`.py`

**Feature flag**：`python-plugin`

**编译命令**：
```bash
cargo build --features python-plugin
```

**运行时依赖**：
- 系统已安装 Python 3.8+（CPython）
- `luo9_sdk` Python 包（位于 `sdk/python/` 目录）

**SDK 搜索路径**（按优先级）：
1. 插件所在目录（支持 `from luo9_sdk import ...`）
2. `{工作目录}/sdk/python/`
3. `{工作目录}/../sdk/python/`

**示例**：
```python
# plugins/my_plugin.py
from luo9_sdk import Bus, Bot, Command, PrefixMode, BusPayload, PayloadType, MsgType

def handle_group_msg(group_id, user_id, msg):
    cmd = Command.parse(msg, "echo", PrefixMode.Required('/'))
    if cmd and not cmd.empty():
        Bot.send_group_msg(group_id, cmd.args_raw())

def plugin_main():
    """插件主函数，由宿主调用"""
    sub = Bus.topic("luo9_message").subscribe()
    print("[my_plugin] Python 插件已启动")

    while True:
        json_str = Bus.topic("luo9_message").pop(sub)
        if json_str is not None:
            payload = BusPayload.parse(json_str)
            if payload and payload.type == PayloadType.MESSAGE:
                if payload.message.message_type == MsgType.GROUP:
                    handle_group_msg(
                        payload.message.group_id or 0,
                        payload.message.user_id,
                        payload.message.message,
                    )
```

**入口约定**：模块顶层必须定义 `plugin_main()` 函数（无参数）

**注意事项**：
- 插件在独立 OS 线程中运行，阻塞调用不影响 tokio 运行时
- SDK 内部通过 ctypes 调用 `luo9_core.dll` FFI，调用 C 函数时自动释放 GIL
- `Bus.topic("luo9_message").pop(sub)` 为非阻塞，返回 `None` 表示无消息
- 宿主禁用插件时，subscriber 被标记为 dead，`pop()` 返回 `None`，循环应据此退出

---

## 3. Java/Kotlin 插件

**文件格式**：`.jar`

**Feature flag**：`java-plugin`

**编译命令**：
```bash
cargo build --features java-plugin
```

**运行时依赖**：
- 系统已安装 JDK/JRE 8+（`JAVA_HOME` 环境变量或 `java` 在 PATH 中）
- `luo9_sdk` Java 包（`sdk/java/` 编译产物）
- JNA 库（插件需包含 `jna.jar` 依赖）

**classpath 组成**（自动构建）：
1. 插件 JAR 文件
2. 插件同目录下所有 `.jar`（依赖库）
3. `{工作目录}/sdk/java/target/*.jar`（SDK 编译产物）

**示例**：
```java
// com/luo9/plugin/MyPlugin.java
package com.luo9.plugin;

import com.luo9.sdk.*;

public class MyPlugin {
    public static void plugin_main() {
        int sub = Bus.topic("luo9_message").subscribe();
        System.out.println("[my_plugin] Java 插件已启动");

        while (true) {
            String json = Bus.topic("luo9_message").pop(sub);
            if (json != null) {
                BusPayload payload = BusPayload.parse(json);
                if (payload != null && payload.type == PayloadType.MESSAGE) {
                    if (payload.message.messageType == MsgType.GROUP) {
                        Bot.sendGroupMsg(payload.message.groupId, "收到消息");
                    }
                }
            }
            try { Thread.sleep(1); } catch (InterruptedException ignored) {}
        }
    }
}
```

**入口约定**：
- 主类：`com.luo9.plugin.PluginMain`（默认查找）
- 方法：`public static void plugin_main()`
- Kotlin 同理，使用 `@JvmStatic fun plugin_main()`

**打包命令**：
```bash
# Maven
mvn package

# Gradle
gradle build

# 手动
jar cf my_plugin.jar -C build/classes .
```

---

## 4. JavaScript 插件（QuickJS）

**文件格式**：`.js`

**Feature flag**：`quickjs-plugin`

**编译命令**：
```bash
cargo build --features quickjs-plugin
```

**运行时依赖**：无（QuickJS 引擎内嵌，零外部依赖）

**SDK 访问方式**：
- 宿主注册 `__luo9_core` 全局对象，包含所有 FFI 函数
- SDK 的 `core.js` 读取 `globalThis.__luo9_core`
- 插件通过 `import` 或直接使用全局对象

**示例**：
```javascript
// plugins/my_plugin.js
// QuickJS 环境下通过 __luo9_core 全局对象访问 FFI

// 手动封装 Bus API（或 import luo9-sdk 模块）
const _core = globalThis.__luo9_core;

const Bus = {
  topic(name) {
    return {
      subscribe() { return _core.luo9_bus_subscribe(name); },
      pop(subId) { return _core.luo9_bus_pop(name, subId); },
      publish(payload) { return _core.luo9_bus_publish(name, payload); },
    };
  },
};

const Bot = {
  sendGroupMsg(groupId, msg) {
    const req = JSON.stringify({
      action: 'send_group_msg',
      group_id: groupId,
      message: msg,
    });
    _core.luo9_bus_publish('luo9_send', req);
  },
};

function plugin_main() {
  const sub = Bus.topic('luo9_message').subscribe();
  console.log('[my_plugin] JavaScript 插件已启动');

  while (true) {
    const json = Bus.topic('luo9_message').pop(sub);
    if (json !== null) {
      const data = JSON.parse(json);
      if (data.Message) {
        const msg = data.Message;
        if (msg.message_type === 'group') {
          Bot.sendGroupMsg(msg.group_id, '收到消息');
        }
      }
    }
  }
}
```

**入口约定**：全局作用域必须定义 `plugin_main()` 函数

**注意事项**：
- QuickJS 是单线程 JS 引擎，插件在独立 OS 线程中运行
- `__luo9_core` 提供的 FFI 函数签名与 `luo9_core.dll` 一致
- 字符串参数/返回值自动在 JS string 和 C string 之间转换
- 返回 `null` 的函数表示无数据或指针为空

---

## 通用配置

### config/default.toml

```toml
[plugin]
# 插件目录（相对路径或绝对路径）
plugin_dir = "plugins"

# 已安装插件列表（由宿主自动管理，也可手动配置）
[[plugins.plugins]]
name = "my_plugin"
priority = 0          # 优先级（越大越先分发）
block_enabled = false  # 启用消息阻断（高优先级插件可阻止后续插件收到消息）

[[plugins.plugins]]
name = "another_plugin"
priority = 10
block_enabled = true
```

### 优先级分发

- 宿主按 `priority` 降序遍历插件，逐个定向推送消息
- 若插件启用 `block_enabled`，则该插件处理后停止分发（后续低优先级插件不再收到此消息）
- 插件代码无需感知优先级机制，SDK 内部透明使用预分配的 subscriber_id

### 插件生命周期

| 操作 | 宿主动作 | 插件感知 |
|------|---------|---------|
| 启动 | 创建 subscriber → spawn 线程 → 调用 `plugin_main()` | 正常运行 |
| 禁用 | `unsubscribe_all()` → 标记 dead → 等待线程退出 | `pop()` 返回 `None` → 循环退出 |
| 热重载 | 禁用 → 重新加载文件 → 启动 | 新实例运行 |
| 优先级变更 | 更新 dispatch_list | 无感知 |
| 阻断开关 | 更新 dispatch_list | 无感知 |

### WebUI 管理

通过 `http://127.0.0.1:27080?token=xxxxx` 访问 WebUI，支持：
- 查看已安装插件列表及运行状态
- 上传/删除插件文件
- 启用/禁用/热重载插件
- 调整插件优先级和消息阻断开关
- 实时查看运行日志
- 插件商店一键安装

---

## Feature Flags 速查

| Feature | 说明 | 依赖 |
|---------|------|------|
| `python-plugin` | 嵌入 CPython 解释器 | `pyo3` |
| `java-plugin` | 嵌入 JVM | `jni` |
| `quickjs-plugin` | 嵌入 QuickJS 引擎 | `rquickjs` |
| `bot_debug` | 心跳/生命周期调试日志 | 无 |
| `plugin_dispatch_debug` | 插件分发调试日志 | 无 |

多个 feature 可同时启用：
```bash
cargo build --features "python-plugin,java-plugin,quickjs-plugin"
```
