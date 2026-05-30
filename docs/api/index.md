# API 参考

本节提供 luo9_bot 的 API 参考文档。

## 目录

- [WebUI API](/api/webui) — HTTP API 端点
- [事件类型](/api/events) — OneBot v11 事件定义

## 快速参考

### WebUI API

| 端点 | 方法 | 功能 |
|---|---|---|
| `/api/status` | GET | 运行状态 |
| `/api/plugins` | GET | 插件列表 |
| `/api/plugins/upload` | POST | 上传插件 |
| `/api/plugins/{name}` | DELETE | 删除插件 |
| `/api/plugins/{name}/enable` | POST | 启用插件 |
| `/api/plugins/{name}/disable` | POST | 禁用插件 |
| `/api/plugins/{name}/reload` | POST | 热重载插件 |
| `/api/logs` | GET | 获取日志 |
| `/api/config` | GET/PUT | 获取/更新配置 |

### 事件类型

| PostType | 说明 |
|---|---|
| `message` | 消息事件 |
| `notice` | 通知事件 |
| `request` | 请求事件 |
| `meta_event` | 元事件 |
