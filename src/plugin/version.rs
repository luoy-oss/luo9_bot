// src/plugin/version.rs
// 插件版本查询：宿主启动后广播查询，收集各插件版本信息

use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

use super::bus;
use super::manager::GLOBAL_PLUGIN_MANAGER;

/// 查询所有插件版本，带超时
///
/// 必须在插件加载完成、插件线程已启动之后调用。
/// 内部流程：
/// 1. 订阅 luo9_version_reply（接收插件响应）
/// 2. 发布 query 到 luo9_version（触发插件响应）
/// 3. 在超时内循环 pop 响应消息
/// 4. 超时后将未响应的插件标记为 "Unknown"
pub async fn query_versions(timeout: Duration) {
    // 1. 先订阅响应 topic，确保不会漏掉任何响应
    let reply_sub = bus::Bus::topic(bus::TOPIC_VERSION_REPLY)
        .subscribe()
        .unwrap_or_else(|e| panic!("订阅 {} topic 失败: {:?}", bus::TOPIC_VERSION_REPLY, e));
    info!("[version] 已订阅版本响应 topic, sub_id={}", reply_sub);

    // 2. 发布查询
    let query = r#"{"action":"query"}"#;
    if let Err(e) = bus::Bus::topic(bus::TOPIC_VERSION).publish(query) {
        error!("[version] 发布版本查询失败: {:?}", e);
        return;
    }
    info!("[version] 已发布版本查询，等待插件响应...");

    // 3. 收集响应（带超时）
    let deadline = Instant::now() + timeout;
    let mut responded = 0u32;

    while Instant::now() < deadline {
        if let Some(json) = bus::Bus::topic(bus::TOPIC_VERSION_REPLY).pop(reply_sub) {
            debug!("[version] 收到响应: {}", json);
            match serde_json::from_str::<serde_json::Value>(&json) {
                Ok(resp) => {
                    let action = resp["action"].as_str().unwrap_or("");
                    if action == "response" {
                        let name = resp["name"].as_str().unwrap_or("").to_string();
                        let version = resp["version"].as_str().unwrap_or("Unknown").to_string();
                        if !name.is_empty() {
                            let mut manager = GLOBAL_PLUGIN_MANAGER.lock().await;
                            manager.update_plugin_version(&name, &version);
                            responded += 1;
                        }
                    }
                }
                Err(e) => warn!("[version] 解析响应失败: {}", e),
            }
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    // 4. 标记未响应的插件
    let total = {
        let mut manager = GLOBAL_PLUGIN_MANAGER.lock().await;
        manager.mark_unknown_versions();
        manager.get_all_plugins().len()
    };

    info!("[version] 版本查询完成: {}/{} 个插件已响应", responded, total);
}
