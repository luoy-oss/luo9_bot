# 介绍

## 概述

luo9_bot 的插件系统基于 FFI 消息总线设计：

1. **核心库（luo9_core）** 提供底层 `extern "C"` 函数
2. **SDK** 封装这些函数，提供各语言的惯用 API
3. **插件** 使用 SDK 编写业务逻辑

## 架构

```
┌─────────────────────────────────────────────────────┐
│                    插件（你的代码）                    │
├─────────────────────────────────────────────────────┤
│                  SDK（语言封装层）                     │
├─────────────────────────────────────────────────────┤
│            luo9_core.dll / libluo9_core.so           │
│                  FFI 接口层                           │
├─────────────────────────────────────────────────────┤
│                   宿主（luo9_bot）                    │
└─────────────────────────────────────────────────────┘
```

## 快速开始

想直接开始写插件？根据你使用的语言选择对应的指南：

- [Rust 插件开发指南](/sdk/rust-plugin-dev) — 官方 SDK，完整支持

## 核心概念

### Bus 消息总线

所有通信通过消息总线进行：

- **Topic**：消息主题，如 `luo9_message`、`luo9_send`
- **Subscriber**：订阅者，每个插件在每个 topic 上有独立的 subscriber
- **Publish**：发布消息到 topic
- **Pop**：从 subscriber 队列取消息

详见 [Bus 消息总线](/sdk/bus)。

### Topic 一览

| Topic | 方向 | 用途 |
|---|---|---|
| `luo9_message` | Host → Plugin | QQ 消息（私聊/群聊） |
| `luo9_meta_event` | Host → Plugin | 元事件（心跳/生命周期） |
| `luo9_notice` | Host → Plugin | 通知事件（好友/群变动等） |
| `luo9_request` | Host → Plugin | 请求事件（好友请求/群请求） |
| `luo9_task` | Host → Plugin | 定时任务事件（tick 下发） |
| `luo9_task_miso` | Plugin → Host | 定时任务请求（schedule/cancel） |
| `luo9_send` | Plugin → Host | 消息发送请求 |
| `luo9_version` | Host → Plugin | 版本查询请求 |
| `luo9_version_reply` | Plugin → Host | 版本查询响应 |

### 插件入口

插件必须导出 `plugin_main` 函数：

```c
extern "C" void plugin_main();
```

宿主在独立线程中调用此函数，panic 会被 `catch_unwind` 捕获。

### 预分配 Subscriber

宿主在加载插件时为每个 topic 创建 subscriber，通过 `luo9_init_subscribers` 传递：

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

SDK 调用 `Bus::topic("luo9_message").subscribe()` 时应优先检查预分配 ID。

## 想开发新 SDK？

如果你希望为其他语言开发 SDK，请查看 [开发新 SDK](/sdk/dev-new-sdk)。
