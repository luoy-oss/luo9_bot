# 配置说明

## 配置文件在哪

洛玖会按这个顺序找配置文件：

1. 环境变量 `LUO9_CONFIG` 指定的路径
2. `~/.luo9/config/default.toml`
3. `config/default.toml`（向后兼容）

找不到会报错，所以至少得有一个。

## 完整配置

```toml
[napcat]
ws_client_host = "127.0.0.1"
ws_client_port = 3001
ws_server_host = "127.0.0.1"
ws_server_port = 23001
timeout_seconds = 30
token = ""

[logging]
level = "info"

[plugins]
enabled = true
plugin_dir = "plugins"
auto_load = true

[[plugins.plugins]]
name = "example_plugin"
priority = 100
block_enabled = false

[webui]
enabled = true
host = "0.0.0.0"
port = 27080
token = ""
```

## 逐项解释

### [napcat]

和 Napcat 的连接配置。

| 字段 | 说明 |
|---|---|
| `ws_client_host` / `ws_client_port` | 接收消息用的 WebSocket 地址 |
| `ws_server_host` / `ws_server_port` | 调 Napcat API 用的地址 |
| `timeout_seconds` | 连接超时 |
| `token` | Napcat 的 access token，没有就留空 |

### [logging]

| 字段 | 说明 |
|---|---|
| `level` | 日志级别：`trace` / `debug` / `info` / `warn` / `error` |

平时用 `info` 就够了。调试的时候可以开 `debug`。

### [plugins]

| 字段 | 说明 |
|---|---|
| `enabled` | 要不要启用插件系统 |
| `plugin_dir` | 插件文件放哪 |
| `auto_load` | 新插件放进去自动加载 |

### [[plugins.plugins]]

每个插件的单独配置。

| 字段 | 说明 |
|---|---|
| `name` | 插件名（不含扩展名） |
| `priority` | 优先级，数字越大越先收到消息 |
| `block_enabled` | 设为 `true` 后，这个插件处理完的消息不会再往下分发 |

### [webui]

| 字段 | 说明 |
|---|---|
| `enabled` | 要不要启用 WebUI |
| `host` | 监听地址，`0.0.0.0` 表示所有网卡 |
| `port` | 监听端口 |
| `token` | 访问密码，留空自动生成 |

## 环境变量

不想改配置文件？环境变量也能覆盖部分设置：

| 变量 | 作用 |
|---|---|
| `LUO9_CONFIG` | 指定配置文件路径 |
| `LUO9_PLUGIN_DIR` | 覆盖插件目录 |

## 在线改配置

WebUI 的「配置」标签页可以直接编辑 TOML 配置，保存即生效。

不过有些配置改了需要重启才能生效，比如 Napcat 的连接地址。
