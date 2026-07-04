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
pub mod request;

pub mod plugin;
pub mod sub_type;
pub mod webui;

use config::LNConfig;
use error::Result;
use tracing::{info, warn};
use tokio::sync::watch;

use plugin as Plugin;

/// 全局重启信号
lazy_static::lazy_static! {
    pub static ref RESTART_TX: watch::Sender<bool> = {
        let (tx, _rx) = watch::channel(false);
        tx
    };
}


/// 应用上下文
pub struct LNContext {
    pub config: LNConfig,
    pub rx: connection::Receiver,
    pub tx: Option<connection::Sender>,
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

        // 尝试连接 Napcat API，失败时不中断程序
        let tx = match connection::Sender::connect(
            &config.napcat.ws_server_host,
            config.napcat.ws_server_port,
            config.napcat.timeout_seconds,
            &config.napcat.token,
        ).await {
            Ok(sender) => {
                info!("✓ Napcat API 连接成功");
                Some(sender)
            }
            Err(e) => {
                warn!("⚠ Napcat API 连接失败: {}", e);
                warn!("WebUI 将继续运行，但插件功能可能不可用");
                None
            }
        };

        // 初始化插件系统（含 bus 总线初始化、插件加载、接收器启动）
        let _ = Plugin::initialize(&config.plugins.plugin_dir, &config.plugins.plugins).await;

        // 初始化 bus 消息发送器（如果有连接）
        if let Some(ref sender) = tx {
            Plugin::sender::init_sender(sender.clone()).await;
        }

        // 启动后台重试任务（如果初始连接失败）
        if tx.is_none() {
            let ws_host = config.napcat.ws_server_host.clone();
            let ws_port = config.napcat.ws_server_port;
            let timeout = config.napcat.timeout_seconds;
            let token = config.napcat.token.clone();

            tokio::spawn(async move {
                info!("启动 WebSocket 后台重试任务...");
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

                    info!("尝试重新连接 Napcat API...");
                    match connection::Sender::connect(&ws_host, ws_port, timeout, &token).await {
                        Ok(sender) => {
                            info!("✓ Napcat API 重新连接成功");
                            // 初始化 bus 消息发送器
                            Plugin::sender::init_sender(sender).await;
                            break;
                        }
                        Err(e) => {
                            warn!("Napcat API 重试失败: {}", e);
                        }
                    }
                }
            });
        }

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

        if let Some(ref tx) = self.tx {
            match tx.get_login_info().await {
                Ok(_info) => {
                    info!("✓ Napcat API 连接正常");
                }
                Err(e) => {
                    tracing::warn!("⚠ Napcat API 连接异常: {}", e);
                    tracing::warn!("请确保 Napcat 已启动并监听 {}:{}",
                        self.config.napcat.ws_server_host,
                        self.config.napcat.ws_server_port);
                }
            }
        } else {
            tracing::warn!("⚠ Napcat API 未连接，插件功能不可用");
            tracing::warn!("请确保 Napcat 已启动并监听 {}:{}",
                self.config.napcat.ws_server_host,
                self.config.napcat.ws_server_port);
        }

        info!("应用已启动，等待消息...");

        Ok(())
    }

    /// 释放资源（用于重启）
    pub async fn shutdown(&self) {
        info!("正在释放资源...");

        // 禁用所有插件
        let mut manager = plugin::GLOBAL_PLUGIN_MANAGER.lock().await;
        let plugins = manager.get_all_plugins().to_vec();
        for plugin_info in plugins {
            if plugin_info.active {
                info!("正在禁用插件: {}", plugin_info.name);
                match manager.disable_plugin(&plugin_info.name, false).await {
                    Ok(msg) => info!("{}", msg),
                    Err(e) => warn!("禁用插件 {} 失败: {}", plugin_info.name, e),
                }
            }
        }
        drop(manager);

        info!("资源释放完成");
    }
}

