//! 洛玖机器人主程序
//! 
//! 这个程序是洛玖机器人的入口点，负责初始化配置、启动服务和处理消息。

use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use axum::{
    routing::post,
    Router, Json, extract::State,
};
use serde_json::{json, Value as JsonValue};
use anyhow::Result;

use luo9_bot::{
    config::{self, Value},
    core::{Driver, Task},
    init_logger,
};

/// 应用状态，包含配置值和驱动实例
/// 实现 Clone 特性以便在多个处理程序间共享
#[derive(Clone)]
struct AppState {
    value: Arc<Value>,
    driver: Arc<Mutex<Driver>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志系统
    init_logger();
    tracing::info!("洛玖机器人启动中...");
    
    let project_path: String = env::current_dir().unwrap().to_str().unwrap().to_string();

    // 加载配置文件
    let config: config::Config = config::load_config(&(project_path + "\\data\\config.yaml"))?;
    let value: Arc<Value> = Arc::new(Value::new(&config));
    
    println!("{:?}", value);
    // 初始化驱动实例并包装在 Arc<Mutex<>> 中以便安全共享
    let driver = Arc::new(Mutex::new(Driver::new(value.clone())));
    
    // 执行驱动的启动流程
    driver.lock().await.run_startup().await?;
    tracing::info!("驱动初始化完成");
    
    // 初始化任务系统
    let task = Arc::new(Mutex::new(Task::new(value.clone(), driver.clone())));

    // 创建应用状态，用于在HTTP处理程序中共享
    let app_state = AppState {
        value: value.clone(),
        driver: driver.clone(),
    };

    // 设置HTTP路由 - 修复泛型参数
    let app = Router::new()
        .route("/", post(receive_event))
        .with_state(app_state);
    
    
    // 从配置中获取服务器地址和端口
    let host = &value.ncc_host;
    let port = value.ncc_port;
    tracing::info!("配置的主机和端口: {}:{}", host, port);
    
    // 修复地址解析问题
    let addr = if host == "localhost" {
        format!("127.0.0.1:{}", port).parse::<std::net::SocketAddr>()?
    } else {
        format!("{}:{}", host, port).parse::<std::net::SocketAddr>()?
    };
    
    tracing::info!("HTTP服务器启动在 {}", addr);
    
    // 并发启动HTTP服务器和任务系统
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let server_future = axum::serve(listener, app);
    
    // 创建一个任务变量，避免临时值问题
    let task_lock = task.lock().await;
    let task_future = task_lock.start();
    
    // 使用tokio::select!来同时运行服务器和任务，并处理中断信号
    tokio::select! {
        result = server_future => {
            let server_result = result;
            if let Err(e) = server_result {
                tracing::error!("服务器错误: {}", e);
            }
        },
        result = task_future => {
            if let Err(e) = result {
                tracing::error!("任务系统错误: {}", e);
            }
        },
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("接收到中断信号，正在关闭...");
        }
    }
    
    // 执行关闭流程
    tracing::info!("正在关闭驱动...");
    driver.lock().await.run_shutdown().await?;
    tracing::info!("洛玖机器人已安全关闭");
    
    Ok(())
}

// /// 处理接收到的事件
// /// 
// /// 这个函数处理来自HTTP请求的事件数据，根据事件类型分发到不同的处理函数
async fn receive_event(
    State(_state): State<AppState>,
    Json(_data): Json<JsonValue>,
) -> Json<JsonValue> {
    println!("{:?}", _data);
    Json(json!({"OK": 200}))
}