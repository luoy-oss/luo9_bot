use tracing_subscriber::fmt;
use tracing_subscriber::filter::EnvFilter;


pub fn init(level: &str) {
    let filter = EnvFilter::new(level);
    
    fmt::Subscriber::builder()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(true)
        .init();
    
    tracing::info!("日志系统已初始化，级别: {}", level);
}