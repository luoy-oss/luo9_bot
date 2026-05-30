---
layout: home

hero:
  name: "洛玖机器人"
  text: "一个让 QQ 机器人开发变得简单的框架"
  tagline: 基于 Napcat OneBot v11，用你喜欢的语言写插件
  actions:
    - theme: brand
      text: 从这里开始
      link: /guide/getting-started
    - theme: alt
      text: 写个插件
      link: /sdk/rust-plugin-dev
    - theme: alt
      text: GitHub
      link: https://github.com/luoy-oss/luo9_bot

features:
  - icon: ⚡
    title: 跑得快
    details: Rust 异步运行时打底，WebSocket 实时通信，消息不过夜
  - icon: 🔌
    title: 插件自由
    details: FFI 消息总线架构，Rust、C++、Python 都能写插件，语言不是限制
  - icon: 🎯
    title: 消息说了算
    details: 优先级分发 + 阻断机制，谁先收到消息、谁能拦住消息，你说了算
  - icon: 🔄
    title: 热重载
    details: 插件更新不用重启机器人，禁用、替换、启用，一气呵成
  - icon: ⏰
    title: 定时任务
    details: 内置 cron 调度器，6 字段表达式，L/W/# 特殊字符都支持
  - icon: 🌐
    title: WebUI
    details: 粉彩风格管理界面，插件管理、日志查看、配置编辑，浏览器里搞定
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
