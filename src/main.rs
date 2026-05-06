use luo9_bot::LNContext;
use tracing::{info, error};
use luo9_bot::handler::core;
use luo9_bot::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
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

    tokio::select! {
        _ = rx_task => {
            info!("接收任务结束");
        }
    }

    Ok(())
}
