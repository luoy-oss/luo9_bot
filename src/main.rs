use luo9_bot::LNContext;
use tracing::{info, error};
use luo9_bot::handler::core;
use luo9_bot::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    loop {
        info!("正在初始化应用...");
        let ctx: LNContext = LNContext::initialize().await?;
        ctx.run().await?;

        // 启动 WebUI
        let webui_cfg = ctx.config.webui.clone();
        let plugin_dir = ctx.config.plugins.plugin_dir.clone();
        if webui_cfg.enabled {
            tokio::spawn(async move {
                luo9_bot::webui::start(&webui_cfg.host, webui_cfg.port, plugin_dir, webui_cfg.token).await;
            });
        }

        let rx = ctx.rx.clone();
        let rx_task = tokio::spawn(async move {
            info!("开始监听 Napcat 推送的消息...");

            loop {
                match rx.receive_one().await {
                    Ok(Some(data)) => {
                        let _ = core::handle(data);
                    }
                    Ok(None) => {
                        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    }
                    Err(e) => {
                        error!("接收消息错误: {}", e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                }
            }
        });

        // 监听重启信号
        let mut restart_rx = luo9_bot::RESTART_TX.subscribe();

        tokio::select! {
            _ = rx_task => {
                info!("接收任务结束");
                break;
            }
            _ = restart_rx.changed() => {
                if *restart_rx.borrow() {
                    info!("收到重启信号，正在重启...");
                    ctx.shutdown().await;
                    // 重置信号
                    luo9_bot::RESTART_TX.send(false).ok();
                    info!("资源已释放，即将重新初始化...");
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    continue;
                }
            }
        }

        break;
    }

    Ok(())
}
