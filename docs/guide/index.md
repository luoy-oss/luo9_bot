# 介绍

**洛玖机器人 (luo9_bot)** 是一个基于 Napcat (OneBot v11) 协议的 QQ 机器人框架，通过 FFI 消息总线支持原生 DLL/SO 插件。

## 核心特性

### 🚀 高性能架构

- **Rust 异步运行时**：基于 tokio，WebSocket 实时通信
- **无阻塞消息处理**：使用 `spawn_blocking` + `wait_pop` 避免阻塞 tokio worker
- **无界队列**：Bus 不丢消息，保证消息可靠性

### 🔌 多语言插件支持

支持三种语言编写插件：

当前官方提供 Rust SDK（完整实现），FFI 接口公开，任何语言均可编写插件。

### 🎯 精确消息控制

- **优先级定向分发**：按优先级降序遍历插件，使用 `publish_to` 定向推送
- **消息阻断**：高优先级插件可启用 `block_enabled` 阻止低优先级插件接收消息
- **Per-topic 并发**：不同 topic 的 publish/pop 互不阻塞

### 🔄 热重载机制

插件支持运行时启用/禁用/热重载，**插件代码零修改**：

1. **加载**：宿主为每个插件预创建 subscriber，通过 `luo9_init_subscribers` FFI 传递 ID
2. **禁用**：调用 `unsubscribe_all()` 标记 dead + 唤醒线程 → 插件循环退出 → 线程结束
3. **热重载**：禁用后重新加载 .dll/.so，创建新 subscriber，spawn 新线程

### ⏰ 定时任务

内置轻量 cron 调度器，支持 6 字段格式：

```
秒 分 时 日 月 周
```

支持的特殊字符：`*` `?` `-` `,` `/` `L` `W` `#`

### 🌐 WebUI 管理

可爱粉彩风格 Web 界面，支持：

- 插件管理（启用/禁用/热重载/优先级/阻断开关）
- 实时日志查看
- 配置编辑
- 插件商店下载

## 技术架构

```
Napcat ──WebSocket──> Receiver (:3001)
                          │
                     handler/core.rs  ← JSON 路由
                     ├── handler/message.rs
                     ├── handler/event.rs
                     ├── handler/notice.rs
                     └── handler/request.rs
                          │
                     plugin::dispatch_*()
                          │
                     FFI Bus (luo9_core.dll)
                     ├── topic: luo9_message
                     ├── topic: luo9_meta_event
                     ├── topic: luo9_notice
                     ├── topic: luo9_request
                     ├── topic: luo9_task_miso ──> plugin/task.rs (cron 调度器)
                     ├── topic: luo9_task       ──> 调度事件下发
                     └── topic: luo9_send ──> plugin/sender.rs ──> Sender (:23001) ──> Napcat API
                          │
                     插件 (DLL/SO，独立线程)
```

## 三层架构

| 层 | 位置 | 职责 |
|---|---|---|
| 宿主 (Host) | `rust/` | WebSocket 连接、事件路由、插件生命周期管理 |
| 核心库 (Core) | `sdk/core/` → `luo9_core.dll` | FFI 消息总线、命令解析，暴露 `extern "C"` 函数 |
| SDK | `sdk/rust/` | 各语言对 Core FFI 的惯用封装 |

## 下一步

- [快速开始](/guide/getting-started) — 安装和运行机器人
- [Rust 插件开发指南](/sdk/rust-plugin-dev) — 用 Rust 编写插件
- [配置说明](/guide/configuration) — 详细配置选项
