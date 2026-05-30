# 配置说明

配置文件采用 TOML 格式，路径按以下优先级查找：

1. 环境变量 `LUO9_CONFIG` 指定的路径
2. `~/.luo9/config/default.toml`
3. `config/default.toml`（向后兼容）

## 完整配置示例

```toml
[napcat]
# Napcat WebSocket 客户端（接收消息）
ws_client_host = "127.0.0.1"
ws_client_port = 3001
# Napcat WebSocket 服务端（发送 API）
ws_server_host = "127.0.0.1"
ws_server_port = 23001
# 连接超时（秒）
timeout_seconds = 30
# Napcat access token（如有）
token = ""

[logging]
# 日志级别：trace / debug / info / warn / error
level = "info"

[plugins]
# 是否启用插件系统
enabled = true
# 插件目录
plugin_dir = "plugins"
# 是否自动加载新插件
auto_load = true

# 插件配置
[[plugins.plugins]]
name = "example_plugin"
priority = 100
block_enabled = false

[[plugins.plugins]]
name = "another_plugin"
priority = 50
block_enabled = true

[webui]
# 是否启用 WebUI
enabled = true
# 监听地址
host = "0.0.0.0"
# 监听端口
port = 27080
# 认证 token（留空则每次启动自动生成）
token = ""
```

## 配置项说明

### [napcat]

| 字段 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `ws_client_host` | String | — | Napcat WebSocket 客户端地址（接收消息） |
| `ws_client_port` | u16 | — | Napcat WebSocket 客户端端口 |
| `ws_server_host` | String | — | Napcat WebSocket 服务端地址（发送 API） |
| `ws_server_port` | u16 | — | Napcat WebSocket 服务端端口 |
| `timeout_seconds` | u64 | — | 连接超时（秒） |
| `token` | String | `""` | Napcat access token |

### [logging]

| 字段 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `level` | String | — | 日志级别 |

### [plugins]

| 字段 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `enabled` | bool | `false` | 是否启用插件系统 |
| `plugin_dir` | String | `"plugins"` | 插件目录路径 |
| `auto_load` | bool | `true` | 是否自动加载新插件 |

### [[plugins.plugins]]

每个插件的独立配置：

| 字段 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `name` | String | — | 插件名称（不含扩展名） |
| `priority` | i32 | `0` | 优先级，数值越大越先处理 |
| `block_enabled` | bool | `false` | 是否启用消息阻断 |

### [webui]

| 字段 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `enabled` | bool | `true` | 是否启用 WebUI |
| `host` | String | `"0.0.0.0"` | 监听地址 |
| `port` | u16 | `27080` | 监听端口 |
| `token` | String | `""` | 认证 token，留空自动生成 |

## 插件优先级

插件按 `priority` 降序处理消息：

```
priority: 100 → 50 → 0 → -100
         高优先级 ──────────> 低优先级
```

当 `block_enabled = true` 时，该插件处理完消息后，低优先级插件将**不会收到**这条消息。

## 环境变量

| 环境变量 | 说明 |
|---|---|
| `LUO9_CONFIG` | 覆盖配置文件路径 |
| `LUO9_PLUGIN_DIR` | 覆盖插件目录 |

## 配置热更新

WebUI 支持在线编辑配置：

1. 访问 WebUI「配置」标签页
2. 编辑 TOML 配置
3. 点击「保存」生效

部分配置需要重启机器人生效（如 `ws_client_host`、`ws_server_port`）。
