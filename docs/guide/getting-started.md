# 快速开始

## 你需要准备什么

- **Rust 工具链**：1.75+，没有的话用 [rustup](https://rustup.rs/) 装一个
- **Napcat**：已经装好并登录了 QQ，开启了 OneBot v11 协议
- **操作系统**：Windows 10+ 或 Linux

## 把代码拉下来

```bash
git clone https://github.com/luoy-oss/luo9_bot.git
cd luo9_bot/rust
```

## 改配置

编辑 `config/default.toml`，把 Napcat 的连接信息填上：

```toml
[napcat]
ws_client_host = "127.0.0.1"
ws_client_port = 3001
ws_server_host = "127.0.0.1"
ws_server_port = 23001
timeout_seconds = 30
token = ""

[plugins]
enabled = true
plugin_dir = "plugins"
auto_load = true

[webui]
enabled = true
host = "0.0.0.0"
port = 27080
token = ""
```

`ws_client_port` 是 Napcat 推送消息用的端口，`ws_server_port` 是调 Napcat API 用的端口。具体填多少看你 Napcat 的配置。

`token` 留空的话，每次启动会自动生成一个随机 token 保护 WebUI。

## 编译运行

```bash
cargo run
```

看到类似这样的日志，说明跑起来了：

```
INFO  luo9_bot 启动中...
INFO  WebSocket 接收器已连接: ws://127.0.0.1:3001
INFO  插件系统初始化完成
INFO  WebUI 启动于 http://127.0.0.1:27080?token=a1b2c3d4e5f67890
INFO  luo9_bot 已就绪
```

日志里的 WebUI 地址复制到浏览器打开，就能看到管理界面了。

## 装个插件试试

### 从 WebUI 装

打开 WebUI，切到「插件商店」标签，选一个装。

### 手动装

把编译好的 `.dll`（Windows）或 `.so`（Linux）扔进 `plugins/` 目录。

### 自己编译

```bash
cd plugin/example/rust
cargo build --release
cp target/release/*.dll ../../plugins/   # Windows
cp target/release/*.so ../../plugins/    # Linux
```

装好后重启机器人，在 QQ 群里发 `/echo 你好` 试试。

## 跑不起来？

**连不上 Napcat**：检查 Napcat 有没有在跑，端口对不对。

**插件没加载**：看日志有没有报错。常见原因：文件放错目录了、扩展名不对。

**WebUI 打不开**：确认 `webui.enabled = true`，端口没被占用。

## 下一步

- [配置说明](/guide/configuration) — 所有配置项详解
- [插件系统](/guide/plugin-system) — 了解插件怎么工作的
- [写个插件](/sdk/rust-plugin-dev) — 10 分钟上手
