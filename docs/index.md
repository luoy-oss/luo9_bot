---
layout: home

hero:
  name: "洛玖机器人"
  text: "基于 Napcat OneBot v11 的 QQ 机器人框架"
  tagline: 轻量、高效、支持多语言插件开发
  actions:
    - theme: brand
      text: 快速开始
      link: /guide/getting-started
    - theme: alt
      text: SDK 开发指南
      link: /sdk/
    - theme: alt
      text: GitHub
      link: https://github.com/luoy-oss/luo9_bot

features:
  - icon: 🚀
    title: 高性能
    details: 基于 Rust 异步运行时，WebSocket 实时通信，无阻塞消息处理
  - icon: 🔌
    title: FFI 插件架构
    details: 基于 FFI 消息总线的原生 DLL/SO 插件，任何语言都能编写插件
  - icon: 🎯
    title: 灵活路由
    details: 优先级定向分发 + 消息阻断机制，精确控制消息流向
  - icon: 🔄
    title: 热重载
    details: 运行时启用/禁用/重载插件，无需重启机器人
  - icon: ⏰
    title: 定时任务
    details: 内置轻量 cron 调度器，支持 6 字段表达式和特殊字符
  - icon: 🌐
    title: WebUI 管理
    details: 可爱风格 Web 界面，插件管理、日志查看、配置编辑
---

<style>
:root {
  --vp-home-hero-name-color: transparent;
  --vp-home-hero-name-background: -webkit-linear-gradient(120deg, #e96d8b 30%, #f9a8d4);
}

:root.dark {
  --vp-home-hero-name-background: -webkit-linear-gradient(120deg, #f9a8d4 30%, #e96d8b);
}
</style>
