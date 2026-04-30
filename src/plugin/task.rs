// src/plugin/task.rs
// luo9_task 总线接收与处理模块

use tracing::{debug, error, info};
use super::task_bus::{Bus, BusError};

const TASK_TOPIC: &str = "luo9_task";
const BUS_CAPACITY: usize = 1024;
const POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);

/// 从 luo9_task 总线接收到的任务请求
#[derive(Debug)]
pub struct TaskRequest {
    pub payload: String,
}

/// 启动 task 总线接收器
pub fn start_task_receiver() {
    tokio::spawn(async {
        if let Err(e) = run_receiver().await {
            error!("task 总线接收器启动失败: {:?}", e);
        }
    });
}

async fn run_receiver() -> Result<(), BusError> {
    Bus::init(BUS_CAPACITY).map_err(|e| {
        error!("luo9_bus_init 失败: {:?}", e);
        e
    })?;

    let topic = Bus::topic(TASK_TOPIC);
    info!("task 总线接收器已启动，监听 topic: {}", TASK_TOPIC);

    loop {
        match topic.pop() {
            Some(payload) => {
                debug!("收到 task 消息: {}", payload);
                let task = TaskRequest { payload };
                handle_task(task);
            }
            None => {
                tokio::time::sleep(POLL_INTERVAL).await;
            }
        }
    }
}

/// 处理 task 请求（空壳，后续补充 cron 定时逻辑）
fn handle_task(_task: TaskRequest) {
    // TODO: 实现 task 处理逻辑
}
