// src/plugin/task.rs
// luo9_task 总线接收与处理模块

use tracing::debug;
use super::bus;

/// 从 luo9_task 总线接收到的任务请求
#[derive(Debug)]
pub struct TaskRequest {
    pub payload: String,
}

/// 启动 task 总线接收器
pub fn start_task_receiver() {
    bus::start_topic_receiver(bus::TOPIC_TASK, |payload| async move {
        debug!("收到 task 消息: {}", payload);
        let task = TaskRequest { payload };
        handle_task(task);
    });
}

/// 处理 task 请求（空壳，后续补充 cron 定时逻辑）
fn handle_task(_task: TaskRequest) {
    // TODO: 实现 task 处理逻辑
}
