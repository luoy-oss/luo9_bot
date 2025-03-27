use std::env;
use std::sync::Arc;
use anyhow::Result;

use luo9_bot::{
    config::{self, Value},
    init_logger,
};


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
    
    Ok(())
}

