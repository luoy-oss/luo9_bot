# WebUI API

## 认证

所有需要认证的 API 都需要携带 token：

- **URL 参数**：`?token=xxxxx`
- **Cookie**：首次验证后自动保存

## 端点

### GET /api/status

获取运行状态。

```json
{ "uptime": 3600, "plugin_count": 5, "plugin_dir": "plugins" }
```

### GET /api/plugins

获取已安装插件列表。

```json
[
  {
    "name": "example_plugin",
    "priority": 100,
    "block_enabled": false,
    "active": true
  }
]
```

### POST /api/plugins/upload

上传插件文件（`multipart/form-data`，字段名 `file`）。

### DELETE /api/plugins/{name}

删除插件文件。

### POST /api/plugins/{name}/enable

启用已禁用的插件。

### POST /api/plugins/{name}/disable

禁用插件（运行时退出 + 文件重命名）。

### POST /api/plugins/{name}/reload

热重载插件（禁用后重新启用）。

### PUT /api/plugins/{name}/priority

设置插件优先级。请求体：`{"priority": 100}`

### PUT /api/plugins/{name}/block

设置消息阻断开关。请求体：`{"block_enabled": true}`

### GET /api/registry

获取可用插件列表（含 tags、sdk_version）。

### POST /api/plugins/install/{name}

从注册表下载安装插件。

### GET /api/download-progress

下载进度 SSE 推送（Server-Sent Events）。

```
data: {"plugin_name":"example","status":"downloading","message":"下载中...","progress":0.5}
data: {"plugin_name":"example","status":"success","message":"安装成功","progress":1.0}
```

### GET /api/mirrors

镜像健康状态。

### GET /api/logs

增量获取日志（`?after=N`）。

### GET /api/config/path

当前使用的配置文件路径。

### GET /api/config

获取当前配置（JSON 格式）。

### PUT /api/config

更新配置（JSON body 合并更新）。

### GET /api/config/raw

获取原始 TOML 配置文本。

### PUT /api/config/raw

更新原始 TOML 配置文本。

## 错误响应

```json
{ "success": false, "message": "错误信息" }
```

| 状态码 | 说明 |
|---|---|
| 200 | 成功 |
| 400 | 请求参数错误 |
| 401 | 未认证 |
| 404 | 资源不存在 |
| 500 | 服务器内部错误 |
