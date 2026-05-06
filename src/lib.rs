#[allow(unused_imports)]
use luo9_sdk;


pub mod config;
pub mod error;
pub mod connection;
pub mod utils;
pub mod handler;

pub mod event;
pub mod message;
pub mod notice;

pub mod plugin;
pub mod sub_type;
pub mod webui;

use config::LNConfig;
use error::Result;
use tracing::info;

use plugin as Plugin;


/// 应用上下文
pub struct LNContext {
    pub config: LNConfig,
    pub rx: connection::Receiver,
    pub tx: connection::Sender,
}
impl LNContext {
    pub async fn initialize() -> Result<Self> {
        let config: LNConfig = LNConfig::load()?;
        utils::logger::init(&config.logging.level);
        

        info!("当前核心版本: luo9_core < {} >", luo9_sdk::Bot::get_version());

        info!("开始初始化应用...");

        let rx = connection::Receiver::new(
            &config.napcat.ws_client_host,
            config.napcat.ws_client_port,
        );

        let tx = connection::Sender::connect(
            &config.napcat.ws_server_host,
            config.napcat.ws_server_port,
            config.napcat.timeout_seconds,
            &config.napcat.token,
        ).await?;

        // 初始化插件系统（含 bus 总线初始化、插件加载、接收器启动）
        let _ = Plugin::initialize(&config.plugins.plugin_dir, &config.plugins.plugins).await;

        // 初始化 bus 消息发送器
        Plugin::sender::init_sender(tx.clone()).await;

        // Plugin::load_all_plugins(&config.plugins);

        info!("应用初始化完成");
        
        Ok(Self {
            config,
            rx,
            tx,
        })
    }
    
    /// 启动应用
    pub async fn run(&self) -> Result<()> {
        info!("启动 Napcat Bridge...");
        
        let rx = self.rx.clone();
        tokio::spawn(async move {
            if let Err(e) = rx.start().await {
                tracing::error!("接收器启动失败: {}", e);
            }
        });
        
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        match self.tx.get_login_info().await {
            Ok(_info) => {
                info!("✓ Napcat API 连接成功");
                // info!("登录信息: {}", serde_json::to_string_pretty(&_info)?);
            }
            Err(e) => {
                tracing::warn!("⚠ Napcat API 连接失败: {}", e);
                tracing::warn!("请确保 Napcat 已启动并监听 {}:{}", 
                    self.config.napcat.ws_server_host,
                    self.config.napcat.ws_server_port);
            }
        }
        
        info!("应用已启动，等待消息...");
        
        Ok(())
    }
}

