# CLAUDE.md — luo9_bot (Rust)

洛玖机器人：基于 Napcat (OneBot v11) 协议的 QQ 机器人，通过 FFI 消息总线支持原生 DLL/SO 插件，并通过嵌入式运行时支持 Python/Java/Kotlin/JavaScript 多语言插件。

## 构建与运行

```bash
# 开发构建
cargo build

# 运行（需先配置 config/default.toml）
cargo run

# Release 极致优化（LTO + 单 codegen unit）
cargo build --release

# Feature flags
cargo build --features bot_debug          # 心跳/生命周期调试日志
cargo build --features plugin_dispatch_debug  # 插件分发调试日志

# 嵌入式运行时（按需启用，未启用不增加依赖）
cargo build --features python-plugin      # 嵌入 CPython (pyo3)
cargo build --features java-plugin        # 嵌入 JVM (jni)
cargo build --features quickjs-plugin     # 嵌入 QuickJS
```

## 架构概览

```
Napcat ──WebSocket──> Receiver (server :27001)
                          │
                     handler/core.rs  ← JSON 路由
                     ├── handler/message.rs
                     ├── handler/event.rs
                     └── handler/notice.rs
                          │
                     plugin::dispatch_*()
                          │
                     FFI Bus (luo9_core.dll)
                     ├── topic: luo9_message
                     ├── topic: luo9_meta_event
                     ├── topic: luo9_notice
                     ├── topic: luo9_task
                     └── topic: luo9_send ──> plugin/sender.rs ──> Sender (client :23001) ──> Napcat API
                          │
                     插件 (PluginRuntime trait)
                     ├── NativeRuntime (DLL/SO，独立线程)
                     ├── PythonRuntime (嵌入 CPython，feature gated)
                     ├── JvmRuntime (嵌入 JVM，feature gated)
                     └── QuickjsRuntime (嵌入 QuickJS，feature gated)
```

### 三层架构

| 层 | 位置 | 职责 |
|---|---|---|
| 宿主 (Host) | `rust/` | WebSocket 连接、事件路由、插件生命周期管理 |
| 核心库 (Core) | `sdk/core/` → `luo9_core.dll` | FFI 消息总线、命令解析，暴露 `extern "C"` 函数 |
| SDK | `sdk/rust/`, `sdk/cpp/`, `sdk/go/`, `sdk/python/`, `sdk/java/`, `sdk/kotlin/`, `sdk/nodejs/` | 各语言对 Core FFI 的惯用封装 |

### 关键设计决策

1. **FFI 消息总线而非 trait 对象**：插件是原生共享库（非 Rust trait），通过 `luo9_core` 的 `extern "C"` 函数进行进程内 pub/sub 通信。这样任何语言（Rust/C++/Python）都能写插件。
2. **PluginRuntime trait 抽象**：`PluginHandle` 持有 `Box<dyn PluginRuntime>`，支持 NativeRuntime（DLL/SO）、PythonRuntime（PyO3）、JvmRuntime（JNI）、QuickjsRuntime（QuickJS）等多种运行时，分发层无需感知具体类型。
2. **插件运行在独立 OS 线程**：每个插件的 `plugin_main()` 在自己的 `std::thread` 中执行，阻塞调用不会影响 tokio 运行时。
3. **总线接收器使用 `spawn_blocking` + `wait_pop`**：避免阻塞 tokio worker 线程，同时不用轮询（无 POLL_INTERVAL）。
4. **无界队列**：Bus 不丢消息，`Bus::init()` 无容量参数。
5. **Per-topic 并发**：Bus 内部使用 `RwLock<HashMap>` + per-topic `Mutex` + per-topic `Condvar`，不同 topic 的 publish/pop 互不阻塞，避免 thundering herd。
6. **`PluginData` 统一序列化格式**：`{"Message": {...}}` / `{"MetaEvent": {...}}` / `{"Notice": {...}}` / `{"Request": {...}}`，SDK 端通过 `BusPayload::parse()` 反序列化。
7. **轻量 cron 调度器**：手写 6 字段解析器（无外部 cron crate），支持 `* ? - , / L W #` 及月/周名称，`OnceLock<UnboundedSender>` + `tokio::select!`（mpsc recv + 1s interval），调度器运行在独立 tokio task 中。
8. **优先级定向分发 + 消息阻断**：宿主通过 `Bus::publish_to()` 向指定 subscriber 定向推送消息（而非广播），按优先级降序遍历插件，若高优先级插件启用 `block_enabled` 则停止分发。**插件代码零修改**——SDK 内部透明使用预分配的 subscriber_id。
9. **Sentinel 机制实现热重载**：`unsubscribe()` 将 subscriber 标记为 dead 并唤醒阻塞线程，`wait_pop()` 识别到哨兵 `__luo9_unsubscribed__` 后返回 `Err(BusError::Unsubscribed)`，插件循环自然退出，线程结束，`Arc<Library>` 释放后 `dlclose` 执行。

### 总线 Topic 一览

| Topic | 方向 | 用途 |
|---|---|---|
| `luo9_message` | Host → Plugin | QQ 消息（私聊/群聊） |
| `luo9_meta_event` | Host → Plugin | 元事件（心跳/生命周期） |
| `luo9_notice` | Host → Plugin | 通知事件（好友/群变动等） |
| `luo9_request` | Host → Plugin | 请求事件（好友请求/群请求） |
| `luo9_task` | Plugin ↔ Host | 定时任务请求/事件 |
| `luo9_send` | Plugin → Host | 消息发送请求 |

### Task 协议

插件发布 task 请求到 `luo9_task` topic，宿主接收并调度：

```json
{"action": "schedule", "task_name": "my_task", "cron": "0 */5 * * * *", "payload": "任意数据"}
```

宿主内含轻量 cron 调度器，到期后向同一 topic 发布事件：

```json
{"event": "tick", "task_name": "my_task", "payload": "任意数据"}
```

插件通过订阅 `luo9_task` 接收事件。区分方式：请求有 `action` 字段，事件有 `event` 字段。

**Cron 表达式**：6 字段格式 `秒 分 时 日 月 周`，每秒检查。支持的特殊字符：

| 字符 | 说明 | 示例 |
|---|---|---|
| `*` | 所有值 | `* * * * * *` |
| `?` | 不指定（日/周互斥） | `0 0 12 15 * ?` |
| `-` | 范围 | `0 0 9-17 * * *` |
| `,` | 列表 | `0 0 0 1,15 * *` |
| `/` | 步长 | `0 */5 * * * *` |
| `L` | 最后（日: 月末; 周: 最后周X） | `0 0 0 L * *` / `0 0 0 * * 5L` |
| `W` | 最近工作日（不跨月） | `0 0 0 15W * *` |
| `#` | 第N个星期X | `0 0 0 * * 2#3`（第3个周一） |

月字段支持 `JAN-DEC`，周字段支持 `SUN-SAT`（0 和 7 都是周日）。

**注意**：插件应在订阅 `luo9_task` **之前**发布 task 请求，否则会通过 latch 机制收到自己的请求。

### 插件生命周期与热重载

插件支持运行时启用/禁用/热重载，**插件代码零修改**：

1. **加载**：宿主为每个插件预创建 subscriber（各 topic），通过 `luo9_init_subscribers` FFI 传递 ID，插件的 SDK 内部透明使用预分配 ID
2. **优先级分发**：宿主按 priority 降序遍历插件，使用 `publish_to` 定向推送，若插件启用 `block_enabled` 则停止分发
3. **禁用**：调用 `unsubscribe_all()` 标记 dead + 唤醒线程 → 插件 `wait_pop` 收到 sentinel `__luo9_unsubscribed__` → 循环退出 → 线程结束 → `Arc<Library>` 释放 → `dlclose`
4. **热重载**：禁用后重新加载 .dll/.so，创建新 subscriber，spawn 新线程

配置通过主配置文件 `[[plugins.plugins]]` 持久化，包含 `name`、`priority`、`block_enabled` 字段。

## 模块结构

```
src/
├── main.rs              # 入口：初始化 LNContext，启动消息循环
├── lib.rs               # LNContext { config, rx, tx }，initialize() / run()
├── config.rs            # LNConfig，从 config/default.toml 加载
├── error.rs             # LNErr (thiserror)，crate 级 Result<T> 别名
├── sub_type.rs          # SubType 枚举（35+ 种 OneBot 子类型），手动 serde 实现
├── connection/
│   ├── receiver.rs      # WebSocket 服务端，接收 Napcat 推送
│   └── sender.rs        # WebSocket 客户端，调用 Napcat API（Bearer 认证）
├── handler/
│   ├── core.rs          # 顶层路由：PostType → message/event/notice/request handler
│   ├── message.rs       # 消息处理 → plugin::dispatch_message()
│   ├── event.rs         # 元事件处理 → plugin::dispatch_meta_event()
│   ├── notice.rs        # 通知处理 → plugin::dispatch_notice()
│   └── request.rs       # 请求处理 → plugin::dispatch_request()
├── event/napcat.rs      # PostType, MetaEvent, Status 类型定义
├── message/napcat.rs    # MsgType, Message, Sender, Anonymous, MessageSegment 完整定义
├── notice/napcat.rs     # NoticeType, Notice, FileInfo, HonorType 完整定义
├── request/napcat.rs    # RequestType, GroupRequestSubType, Request 定义
├── plugin/
│   ├── mod.rs           # 插件系统入口：initialize(), priority_dispatch_*()
│   ├── bus.rs           # FFI 总线封装，topic 常量，start_topic_receiver()
│   ├── manager.rs       # PluginManager：生命周期管理、启用/禁用/热重载
│   ├── handle.rs        # PluginHandle：运行时句柄（Box<dyn PluginRuntime>）
│   ├── loader.rs        # PluginLoader：扫描 .dll/.so/.py/.jar/.js，按类型创建 Runtime
│   ├── runtime.rs       # PluginRuntime trait：start/stop/is_alive 抽象
│   ├── native_runtime.rs # NativeRuntime：DLL/SO 运行时（libloading + plugin_main 线程）
│   ├── embedded/
│   │   ├── mod.rs       # 嵌入式运行时模块（feature gated）
│   │   ├── python.rs    # PythonRuntime：PyO3 嵌入 CPython
│   │   ├── jvm.rs       # JvmRuntime：JNI 嵌入 JVM
│   │   └── quickjs.rs   # QuickjsRuntime：嵌入 QuickJS
│   ├── dispatch.rs      # 优先级分发：DISPATCH_LIST + publish_to 定向推送 + 阻断
│   ├── data.rs          # PluginData 枚举（Message | MetaEvent | Notice | Request）
│   ├── sender.rs        # luo9_send 接收器，路由发送请求到 Sender
│   └── task.rs          # luo9_task 接收器 + 6 字段 cron 调度器（支持 ? L W #）
├── webui.rs             # WebUI：axum HTTP 服务，插件管理/日志查看/文件上传
├── webui/
│   ├── index.html       # WebUI 主页面
│   ├── style.css        # 可爱风格样式（粉彩配色 + 动效）
│   └── app.js           # 前端逻辑（标签切换/插件管理/日志查看）
└── utils/logger.rs      # tracing-subscriber 初始化 + 全局日志缓冲区（供 WebUI）
```

## 开发规范

- **语言**：代码注释、commit message、日志输出、错误信息均使用中文
- **Commit 风格**：部分遵循 conventional commits（`refactor(plugin):`, `feat(...):`, `fix(...):`），中文描述为主
- **错误处理**：主体使用 `thiserror` 的 `LNErr` + `Result<T>` 别名；插件系统用 `Box<dyn std::error::Error>` 和 `String`
- **全局状态**：使用 `lazy_static!` + `tokio::sync::Mutex`（如 `GLOBAL_PLUGIN_MANAGER`, `GLOBAL_SENDER`）
- **异步运行时**：tokio (full features)，WebSocket 用 tokio-tungstenite
- **日志**：tracing + tracing-subscriber (env-filter)，通过自定义 `MakeWriter` 同时写入 stdout 和全局缓冲区

## WebUI

默认端口 27080，配置见 `[webui]` 段。多文件结构（`include_str!` 嵌入），可爱粉彩风格 + 动效。

### Token 鉴权

WebUI 支持 token 鉴权保护：

- **配置 token**：在 `[webui]` 段设置 `token = "your_token"`，该 token 永久有效
- **自动生成**：若未配置 token（默认为空），每次启动时自动生成随机 token
- **访问方式**：`http://host:port?token=xxxxx`
- **Cookie 持久化**：首次通过 URL 参数验证后，token 保存到 cookie，后续访问无需重复携带
- **启动日志**：启动时会输出完整访问地址（含 token）

示例输出：
```
INFO  WebUI 启动于 http://127.0.0.1:27080?token=a1b2c3d4e5f67890
```

### API 端点

| 端点 | 方法 | 功能 | 鉴权 |
|---|---|---|---|
| `/` | GET | 主页面（index.html） | 否 |
| `/style.css` | GET | 样式文件 | 否 |
| `/app.js` | GET | 前端脚本 | 否 |
| `/api/status` | GET | 运行时间、插件数量、插件目录 | 是 |
| `/api/plugins` | GET | 已安装插件列表（含 priority、block_enabled、active） | 是 |
| `/api/plugins/upload` | POST | 上传插件文件（multipart） | 是 |
| `/api/plugins/{name}` | DELETE | 删除插件文件 | 是 |
| `/api/plugins/{name}/enable` | POST | 启用已禁用的插件 | 是 |
| `/api/plugins/{name}/disable` | POST | 禁用插件（运行时退出 + 文件重命名） | 是 |
| `/api/plugins/{name}/reload` | POST | 热重载插件（禁用后重新启用） | 是 |
| `/api/plugins/{name}/priority` | PUT | 设置插件优先级（JSON body: `{priority: N}`） | 是 |
| `/api/plugins/{name}/block` | PUT | 设置消息阻断开关（JSON body: `{block_enabled: bool}`） | 是 |
| `/api/plugins/install/{name}` | POST | 从注册表下载安装插件 | 是 |
| `/api/registry` | GET | 获取可用插件列表（含 tags、sdk_version） | 是 |
| `/api/logs` | GET | 增量获取日志（`?after=N`） | 是 |
| `/api/config/path` | GET | 当前使用的配置文件路径 | 是 |
| `/api/config` | GET | 获取当前配置（JSON 格式） | 是 |
| `/api/config` | PUT | 更新配置（JSON body 合并更新） | 是 |
| `/api/config/raw` | GET | 获取原始 TOML 配置文本 | 是 |
| `/api/config/raw` | PUT | 更新原始 TOML 配置文本 | 是 |
| `/api/download-progress` | GET | 下载进度 SSE 推送（Server-Sent Events） | 是 |
| `/api/mirrors` | GET | 镜像健康状态（可用镜像列表、延迟） | 是 |

### 插件商店镜像

注册表获取和插件下载支持镜像健康检查 + fallback 机制：

- **并行健康检查**：启动时并行测试所有镜像连通性和延迟
- **缓存可用镜像**：按延迟排序缓存可用镜像列表（5 分钟刷新）
- **优先使用可用镜像**：下载时直接使用已验证的镜像，无需 fallback
- **回退策略**：缓存未命中时使用默认镜像列表，仍保留 fallback
- **所有 URL 均使用 HTTPS**

### 下载进度推送

插件下载支持实时进度推送（SSE）：

- **SSE 端点**：`/api/download-progress?token=xxxxx`
- **进度事件**：`DownloadProgress { plugin_name, status, message, progress }`
- **status 值**：`downloading`（下载中）、`success`（成功）、`error`（失败）
- **progress 范围**：0.0 - 1.0，`null` 表示不确定
- **前端显示**：右下角浮动面板，显示进度条和状态消息

## 技术债务

1. **测试覆盖不足**：仅 webui 模块有 17 个单元测试，其余模块无测试（core 层有 47 个测试）
2. **错误处理不统一**：三种模式并存——`LNErr`/`Box<dyn Error>`/`String`，应统一为 `LNErr`
3. **`anyhow` 未使用**：Cargo.toml 声明了依赖但代码中从未引用
4. **`plugins` feature 未生效**：插件系统始终初始化，不受 feature gate 控制
5. **硬编码配置路径**：`config/default.toml` 路径写死，且包含绝对路径（`plugin_dir`）
6. **无优雅关闭**：bus 接收器循环无退出机制，无 shutdown signal
7. **无 rustfmt/clippy 配置**：缺乏统一的代码风格约束

## FFI 接口约定

插件必须导出 `plugin_main` 函数：

```c
extern "C" fn plugin_main()  // 在独立线程中运行，panic 被 catch_unwind 捕获
```

`luo9_core` 暴露的核心 FFI 函数：

| 函数 | 用途 |
|---|---|
| `luo9_bus_init()` | 初始化总线单例 |
| `luo9_bus_subscribe(topic)` | 订阅 topic，返回 subscriber_id |
| `luo9_bus_unsubscribe(topic, sub_id)` | 取消订阅，唤醒阻塞线程（sentinel 机制） |
| `luo9_bus_publish(topic, payload)` | 广播消息给所有 subscriber |
| `luo9_bus_publish_to(topic, payload, ids, len)` | 定向推送消息给指定 subscriber |
| `luo9_bus_pop(topic, sub_id)` | 非阻塞取消息 |
| `luo9_bus_wait_pop(topic, sub_id)` | 阻塞取消息（dead subscriber 返回 sentinel） |
| `luo9_bus_free_string(ptr)` | 释放 bus 返回的字符串 |

## 依赖关系

```
luo9_bot (宿主)
  └── luo9_sdk (sdk/rust/)  ← 提供 Bus, Bot, Command 等 FFI 封装
        └── 链接 luo9_core (sdk/core/, cdylib)  ← FFI 实现层（仅依赖 libc）
```

Core 层极简：仅 `libc` 一个依赖。`serde`/`serde_json`/`libloading` 等业务依赖由宿主自行引入。

`luo9_sdk` 通过 local path 依赖：`H:\github\luo9_bot\sdk\rust`（见 Cargo.toml）。

## 多语言 SDK

| 语言 | SDK 位置 | 插件样例 | 状态 |
|---|---|---|---|
| Rust | `sdk/rust/` | `plugin/example/rust/` | 完整（Bus + Command + Payload + Bot + Msg） |
| C++ | `sdk/cpp/` | `plugin/example/cpp/` | 完整（Bus + Command + Payload + Bot + CommandMatcher） |
| Python | `sdk/python/` | `plugin/example/python/` | 完整（Bus + Command + Payload + Bot） |

所有 SDK 封装同一份 `luo9_core.dll` / `libluo9_core.so` 的 FFI 接口。插件样例均实现 `/echo`、`/task start`、`/task end` 指令。

## OneBot v11 事件支持

本项目全量适配 Napcat OneBot v11 协议，支持以下事件类型：

### 消息事件 (PostType::Message / PostType::MessageSent)

| 字段 | 类型 | 说明 |
|---|---|---|
| `message_id` | u64 | 消息 ID |
| `message_seq` | Option<u64> | 消息序列号 |
| `real_id` | Option<u64> | 真实消息 ID |
| `real_seq` | Option<String> | 真实序列号 |
| `message_type` | MsgType | `Private` / `Group` |
| `sub_type` | SubType | `Friend` / `GroupTemp` / `Normal` / `Anonymous` 等 |
| `user_id` | u64 | 发送者 QQ 号 |
| `group_id` | Option<u64> | 群号（群消息时存在） |
| `message` | Vec<MessageSegment> | 结构化消息段 |
| `raw_message` | String | 原始消息文本（已解码 HTML 实体） |
| `font` | u32 | 字体 |
| `sender` | Sender | 发送者信息（含 nickname, card, sex, age, area, level, role, title） |
| `anonymous` | Option<Anonymous> | 匿名信息（群匿名消息时存在） |
| `message_format` | String | 消息格式（array/string） |

### 通知事件 (PostType::Notice)

| NoticeType | 触发场景 | 关键字段 |
|---|---|---|
| `FriendAdd` | 好友添加 | `user_id` |
| `FriendRecall` | 好友消息撤回 | `user_id`, `message_id` |
| `FriendPoke` | 好友戳一戳 | `user_id`, `target_id` |
| `GroupAdmin` | 群管理员变动 | `group_id`, `user_id`, `sub_type`(Set/Unset) |
| `GroupBan` | 群禁言 | `group_id`, `user_id`, `operator_id`, `duration`, `sub_type`(Ban/LiftBan) |
| `GroupIncrease` | 群成员增加 | `group_id`, `user_id`, `operator_id`, `sub_type`(Approve/Invite) |
| `GroupDecrease` | 群成员减少 | `group_id`, `user_id`, `operator_id`, `sub_type`(Leave/Kick/KickMe) |
| `GroupCard` | 群名片修改 | `group_id`, `user_id`, `card_new`, `card_old` |
| `GroupRecall` | 群消息撤回 | `group_id`, `user_id`, `operator_id`, `message_id` |
| `GroupUpload` | 群文件上传 | `group_id`, `user_id`, `file`(FileInfo) |
| `GroupTitle` | 群头衔变更 | `group_id`, `user_id`, `title` |
| `Honor` | 群荣誉变更 | `group_id`, `user_id`, `honor_type`(Talkative/Performer/Emotion) |
| `Essence` | 精华消息 | `group_id`, `user_id`, `message_id` |
| `Poke` | 戳一戳（群内） | `group_id`, `user_id`, `target_id` |
| `LuckyKing` | 运气王 | `group_id`, `user_id`, `target_id` |
| `GroupMsgEmojiLike` | 表情回应（NapCat扩展） | `group_id`, `user_id`, `message_id` |
| `Notify` | 其他通知 | 根据 `sub_type` 区分 |

### 请求事件 (PostType::Request)

| RequestType | 字段 | 说明 |
|---|---|---|
| `Friend` | `user_id`, `comment`, `flag` | 好友请求 |
| `Group` | `group_id`, `user_id`, `comment`, `flag`, `sub_type`(Add/Invite) | 群请求 |

### 元事件 (PostType::MetaEvent)

| MetaEventType | 字段 | 说明 |
|---|---|---|
| `Lifecycle` | `sub_type`(Enable/Disable/Connect) | 生命周期事件 |
| `Heartbeat` | `status`(online, good), `interval` | 心跳事件 |

### SubType 完整列表

**Lifecycle**: Enable, Disable, Connect
**Message**: Friend, GroupTemp, GroupSelf, Other, Normal, Anonymous, Notice
**Notice 管理**: Set, Unset
**Notice 禁言**: Ban, LiftBan, Unban
**Notice 成员**: Leave, Kick, KickMe, Approve, Invite, Add
**Notice 互动**: Poke, LuckyKing
**Notice 荣誉**: Talkative, Performer, Emotion, Honor
**Notice 其他**: InputStatus, Title, ProfileLike
