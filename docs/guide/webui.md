# WebUI

## 概述

luo9_bot 内置可爱粉彩风格的 Web 管理界面，支持插件管理、日志查看、配置编辑等功能。

## 访问

启动机器人后，日志会输出访问地址：

```
INFO  WebUI 启动于 http://127.0.0.1:27080?token=a1b2c3d4e5f67890
```

在浏览器中打开该地址即可访问。

## Token 鉴权

- **配置 token**：在 `[webui]` 段设置 `token = "your_token"`，永久有效
- **自动生成**：未配置时每次启动自动生成随机 token
- **Cookie 持久化**：首次通过 URL 参数验证后，token 保存到 cookie

## 功能标签页

### 🏠 首页

运行状态概览：运行时间、插件数量、插件目录路径。

### 🔌 插件管理

| 功能 | 说明 |
|---|---|
| 启用/禁用 | 运行时启用或禁用插件 |
| 热重载 | 禁用后重新加载插件 |
| 优先级调整 | 设置插件处理消息的优先级 |
| 阻断开关 | 是否阻止低优先级插件接收消息 |
| 上传插件 | 上传 .dll/.so 文件 |
| 删除插件 | 删除插件文件 |

### 📝 日志查看

实时查看机器人日志，支持增量加载和自动滚动。

### ⚙️ 配置编辑

在线编辑 TOML 配置文件，保存后自动生效。

### 🏪 插件商店

从注册表下载安装插件，支持镜像健康检查和下载进度实时推送。

## API 端点

| 端点 | 方法 | 功能 | 鉴权 |
|---|---|---|---|
| `/` | GET | 主页面 | 否 |
| `/api/status` | GET | 运行状态 | 是 |
| `/api/plugins` | GET | 插件列表 | 是 |
| `/api/plugins/upload` | POST | 上传插件 | 是 |
| `/api/plugins/{name}` | DELETE | 删除插件 | 是 |
| `/api/plugins/{name}/enable` | POST | 启用插件 | 是 |
| `/api/plugins/{name}/disable` | POST | 禁用插件 | 是 |
| `/api/plugins/{name}/reload` | POST | 热重载插件 | 是 |
| `/api/plugins/{name}/priority` | PUT | 设置优先级 | 是 |
| `/api/plugins/{name}/block` | PUT | 设置阻断开关 | 是 |
| `/api/registry` | GET | 插件商店列表 | 是 |
| `/api/plugins/install/{name}` | POST | 从商店安装 | 是 |
| `/api/logs` | GET | 获取日志 | 是 |
| `/api/config` | GET/PUT | 获取/更新配置 | 是 |
| `/api/config/raw` | GET/PUT | 原始 TOML | 是 |
| `/api/download-progress` | GET | 下载进度 SSE | 是 |
| `/api/mirrors` | GET | 镜像状态 | 是 |

详细 API 文档见 [WebUI API](/api/webui)。

## 配置

```toml
[webui]
enabled = true
host = "127.0.0.1"
port = 27080
token = ""  # 留空自动生成
```

## 安全建议

1. **生产环境**：务必设置固定 token
2. **外网访问**：建议使用反向代理（如 Nginx）并启用 HTTPS
3. **访问控制**：限制监听地址为 `127.0.0.1`，仅本地访问
