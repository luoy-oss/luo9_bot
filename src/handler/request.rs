use crate::error::Result;
use crate::request::Request;
use tracing::info;

#[cfg(feature = "napcat")]
pub fn handle_request(request: Request) -> Result<()> {
    info!("收到请求: {:?}", request);

    // 请求事件会通过插件系统分发
    crate::plugin::dispatch_request(request);

    Ok(())
}
