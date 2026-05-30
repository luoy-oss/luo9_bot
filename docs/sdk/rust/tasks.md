# 定时任务

## 创建任务

向 `luo9_task_miso` topic 发送请求：

```rust
use luo9_sdk::bus::Bus;
use serde_json::json;

let req = json!({
    "action": "schedule",
    "task_name": "daily_report",
    "cron": "0 0 9 * * *",    // 每天早上 9 点
    "payload": "日报时间到"
});
Bus::topic("luo9_task_miso").publish(&req.to_string()).unwrap();
```

Cron 表达式是 6 字段格式：`秒 分 时 日 月 周`。

一些例子：
- `0 */5 * * * *` — 每 5 分钟
- `0 0 9 * * *` — 每天早上 9 点
- `0 0 0 * * 1` — 每周一午夜
- `0 30 8 * * 1-5` — 工作日早上 8:30

## 接收任务事件

任务触发后，你会从 `luo9_task` 收到事件：

```json
{"event": "tick", "task_name": "daily_report", "payload": "日报时间到"}
```

在插件里这样处理：

```rust
let task_sub = Bus::topic("luo9_task").subscribe().unwrap();
let task_topic = Bus::topic("luo9_task");

// 在循环里
if let Some(json) = task_topic.pop(task_sub) {
    if let Ok(event) = serde_json::from_str::<serde_json::Value>(&json) {
        if event["event"] == "tick" {
            let name = event["task_name"].as_str().unwrap_or("");
            let payload = event["payload"].as_str().unwrap_or("");
            // 执行任务逻辑...
        }
    }
}
```

## 取消任务

```rust
let cancel = json!({
    "action": "cancel",
    "task_name": "daily_report"
});
Bus::topic("luo9_task_miso").publish(&cancel.to_string()).unwrap();
```

## 注意事项

- 任务请求发到 `luo9_task_miso`，任务事件从 `luo9_task` 接收
- 建议在订阅 `luo9_task` **之前**发布任务请求，避免通过 latch 机制收到自己的请求

## 完整示例

```rust
use luo9_sdk::bus::Bus;
use luo9_sdk::payload::*;
use serde_json::json;

#[unsafe(no_mangle)]
pub extern "C" fn plugin_main() {
    // 先发布任务请求
    let req = json!({
        "action": "schedule",
        "task_name": "heartbeat",
        "cron": "0 */1 * * * *",  // 每分钟
        "payload": "ping"
    });
    Bus::topic("luo9_task_miso").publish(&req.to_string()).unwrap();

    // 再订阅任务事件
    let task_sub = Bus::topic("luo9_task").subscribe().unwrap();
    let task_topic = Bus::topic("luo9_task");

    loop {
        if let Some(json) = task_topic.pop(task_sub) {
            if let Ok(event) = serde_json::from_str::<serde_json::Value>(&json) {
                if event["event"] == "tick" {
                    eprintln!("定时任务触发: {}", event["task_name"]);
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
```
