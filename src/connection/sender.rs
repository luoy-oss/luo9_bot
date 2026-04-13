use std::sync::Arc;

use crate::error::Result;
use futures_util::{SinkExt, StreamExt, lock::Mutex, stream::{SplitSink, SplitStream}};
use serde_json::{json, Value};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::{client::IntoClientRequest, http::HeaderValue, protocol::Message}};
use tracing::{info, error};
use crate::error::LNErr;

/// WebSocket 发送器 - 作为客户端连接 Napcat API
#[derive(Clone)]
pub struct Sender {
    write: Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>,
    read: Arc<Mutex<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
    timeout_seconds: u64,
}


impl Sender {
    pub async fn connect(host: impl Into<String>, port: u16, timeout_seconds: u64, token: impl Into<String>) -> Result<Self> {
        let host = host.into();
        let token = token.into();
        let url = format!("ws://{}:{}", host, port);
        
        info!("连接 Napcat API: {} timeout_seconds： {} token: {}", url, timeout_seconds, token);
        let mut request = url.clone().into_client_request()?;
        
        // 建立连接
        // 添加Header 添加参数 Authorization，其值为在 Bearer 之后拼接 Token
        request.headers_mut().insert("Authorization", 
        HeaderValue::from_str(&format!("Bearer {}", token))
            .map_err(|e| LNErr::InvalidHeaderValue(format!("Authorization 头值错误: {}", e)))?);
        
        let (ws_stream, _) = connect_async(request).await?;
        let (write, read) = ws_stream.split();
        
         Ok(Self {
            write: Arc::new(Mutex::new(write)),
            read: Arc::new(Mutex::new(read)),
            timeout_seconds,
        })
    }
    
    /// 发送 API 调用
    pub async fn send_api(&self, action: &str, params: Value) -> Result<Value> {
        // info!("action:{}\tparams:{}", action, params);
        
        let request: Value = json!({
            "action": action,
            "params": params,
            "echo": format!("{}_{}", action, chrono::Utc::now().timestamp())
        });
        
        let request_str = serde_json::to_string(&request)?;
        
        // 发送
        let mut write = self.write.lock().await;
        write.send(Message::Text(request_str.into())).await?;
        
        let mut read = self.read.lock().await;

        // 等待响应（带超时）
        let timeout = tokio::time::Duration::from_secs(self.timeout_seconds);
        match tokio::time::timeout(timeout, read.next()).await {
            Ok(Some(Ok(msg))) => {
                match msg {
                    Message::Text(text) => {
                        let response: Value = serde_json::from_str(&text)?;
                        Ok(response)
                    }
                    Message::Binary(bin) => {
                        let response: Value = serde_json::from_slice(&bin)?;
                        Ok(response)
                    }
                    _ => {
                        Err(LNErr::Config("未收到有效响应".into()))
                    }
                }
            }
            Ok(Some(Err(e))) => {
                error!("WebSocket 接收错误: {}", e);
                Err(LNErr::Config(format!("WebSocket 接收错误: {}", e)))
            }
            Ok(None) => {
                Err(LNErr::Config("连接已关闭".into()))
            }
            Err(_) => {
                Err(LNErr::Config(format!("请求超时 ({}秒)", self.timeout_seconds)))
            }
        }
    }
    
    /// 获取登录信息（测试连接用）
    pub async fn get_login_info(&self) -> Result<Value> {
        self.send_api("get_login_info", json!({})).await
    }

    /// 发送私聊消息
    pub async fn send_private_message(&self, user_id: u64, message: &str) -> Result<Value> {
        let params = json!({
            "user_id": user_id,
            "message": message
        });
        self.send_api("send_private_msg", params).await
    }

    /// 发送群消息
    pub async fn send_group_message(&self, group_id: u64, message: &str) -> Result<Value> {
        let params = json!({
            "group_id": group_id,
            "message": message
        });
        self.send_api("send_group_msg", params).await
    }
}