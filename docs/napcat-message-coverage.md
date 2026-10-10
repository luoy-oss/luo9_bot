# NapCat / OneBot 消息覆盖说明

## 目标与兼容策略

宿主层不能只保留 `raw_message`：NapCat 的结构化 `message` 数组才是图像、语音、视频、文件、回复、转发、JSON/XML 卡片以及未来扩展消息段的可靠载体。本次修改采用双层兼容策略：

- 保留旧的 `Message.message` 字段，继续提供解码后的文本/兼容字符串，避免现有插件立即失效。
- 新增 `Message.raw_message`，保留协议原文。
- 新增 `Message.message_segments: Vec<serde_json::Value>`，不把消息段限制为宿主预定义枚举，未知类型及嵌套字段原样保留。
- 新增 `raw_event` 到消息、通知、请求和元事件结构，保留暂未建模的字段。

## 覆盖分层

| 优先级 | 覆盖目标 | 宿主侧策略 |
| --- | --- | --- |
| P0 | 文本、@、回复、图片、语音、视频、文件等常见消息段；私聊/群聊基础事件 | 结构化消息段原样透传，保留旧消息字段 |
| P1 | 混合消息段、转发节点、JSON/XML 卡片、消息段附加数据 | 使用任意 JSON 数据承载，不因未识别字段而丢失 |
| P2 | NapCat 扩展消息段、通知/请求扩展字段 | 保留完整原始事件，已识别字段继续提供便利结构 |
| P3 | 未来新增的段类型和事件字段 | 不要求宿主先升级模型即可经 `raw_event` / `message_segments` 读取 |

## 插件消费示例

```rust
// 旧插件仍可使用
let legacy_text = &message.message;

// 需要保留 CQ/协议原文时使用
let raw = &message.raw_message;

// 处理结构化消息段；不要假定 type 只包含已知枚举
for segment in &message.message_segments {
    let segment_type = segment.get("type").and_then(|v| v.as_str()).unwrap_or("");
    let data = segment.get("data");
    // 根据 segment_type 选择处理方式；未知类型也可记录或透传
}

// 新增字段可从完整事件中读取
let future_field = message.raw_event.get("future_napcat_field");
```

## 验证清单

- [x] 字符串形式的 message_id 兼容解析。
- [x] 缺少结构化 message 数组时安全退化为空数组。
- [x] 任意/未知消息段及嵌套字段保留。
- [x] 原始 raw_message 与兼容 message 字段分别保留。
- [ ] 在 CI 中运行格式检查、单元测试和 Clippy。
- [ ] 使用真实 NapCat WebSocket 样本验证图片、语音、视频、文件、合并转发、JSON/XML 卡片及扩展通知。
- [ ] 联调 SDK 侧事件模型，确认所有插件 ABI/序列化消费方均兼容新增字段。

> “全量支持”在这里指宿主不丢弃协议数据并可让插件自行处理未知类型，不代表每一种消息段都已实现专用的业务解析或发送 API。
