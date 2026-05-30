# 快速开始

## 环境要求

- **Rust**: 1.75+（推荐使用 [rustup](https://rustup.rs/) 安装）
- **Napcat**: 已安装并配置好 QQ 登录，开启 OneBot v11 协议
- **操作系统**: Windows 10+ / Linux (x86_64)

## 第一步：获取代码

```bash
git clone https://github.com/luoy-oss/luo9_bot.git
cd luo9_bot/rust
```

## 第二步：配置

编辑 `config/default.toml`，填写 Napcat 连接信息：

```toml
[napcat]
# Napcat WebSocket 服务端地址
ws_client_host = "127.0.0.1"
ws_client_port = 3001
# Napcat API 端口
ws_server_host = "127.0.0.1"
ws_server_port = 23001
# access token（如有）
token = ""

[plugins]
enabled = true
plugin_dir = "plugins"
auto_load = true

[webui]
enabled = true
host = "127.0.0.1"
port = 27080
token = ""  # 留空则自动生成
```

## 第三步：构建

```bash
# 开发构建
cargo build

# Release 极致优化（LTO + 单 codegen unit，推荐生产环境）
cargo build --release
```

### Feature Flags

```bash
# 心跳/生命周期调试日志
cargo build --features bot_debug

# 插件分发调试日志
cargo build --features plugin_dispatch_debug
```

## 第四步：运行

```bash
cargo run
```

启动成功后，你将看到类似输出：

```
INFO  luo9_bot 启动中...
INFO  WebSocket 接收器已连接: ws://127.0.0.1:3001
INFO  插件系统初始化完成
INFO  WebUI 启动于 http://127.0.0.1:27080?token=a1b2c3d4e5f67890
INFO  luo9_bot 已就绪
```

## 第五步：安装插件

### 方式一：通过 WebUI 安装

1. 打开浏览器访问启动日志中的 WebUI 地址
2. 切换到「插件商店」标签
3. 选择需要的插件，点击「安装」

### 方式二：手动安装

将编译好的 `.dll`（Windows）或 `.so`（Linux）文件放入 `plugins/` 目录。

### 方式三：从源码编译

```bash
cd plugin/example/rust
cargo build --release
cp target/release/*.dll ../../plugins/  # Windows
cp target/release/*.so ../../plugins/   # Linux
```

## 第六步：验证

在 QQ 群中发送消息，观察机器人是否响应。如果安装了示例插件，可以尝试：

```
/echo 你好世界
```

## 常见问题

### 连接 Napcat 失败

检查 Napcat 是否正常运行，以及 WebSocket 端口是否与配置一致。

### 插件未加载

1. 确认插件文件位于 `plugins/` 目录
2. 确认插件文件扩展名正确（`.dll` 或 `.so`）
3. 检查日志中是否有加载错误信息

### WebUI 无法访问

1. 确认 `webui.enabled = true`
2. 检查端口是否被占用
3. 确认防火墙允许对应端口

## 下一步

- [配置说明](./configuration.md) — 了解所有配置项
- [插件系统](./plugin-system.md) — 了解插件工作原理
- [Rust 插件开发指南](/sdk/rust-plugin-dev) — 用 Rust 编写插件
- [WebUI](./webui.md) — 使用 Web 界面管理机器人
