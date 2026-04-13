use crate::error::Result;
use futures_util::StreamExt;
use serde_json::Value;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};
use tracing::{info, error};

/// WebSocket 接收器 - 作为服务器接收 Napcat 推送的消息
#[derive(Clone)]
pub struct Receiver {
    host: String,
    port: u16,
    // 保存连接的客户端
    clients: Arc<Mutex<Vec<tokio_tungstenite::WebSocketStream<TcpStream>>>>,
}

impl Receiver {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        let host = host.into();
        Self {
            host,
            port,
            clients: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn start(&self) -> Result<()> {
        self.napcat_start().await
    }
    
    /// 启动服务器
    pub async fn napcat_start(&self) -> Result<()> {
        let addr = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(&addr).await?;
        
        info!("Napcat 接收器已启动: ws://{}", addr);
        info!("等待 Napcat 连接... 最长等待时间与你在napcat配置的心跳时间相同（默认30秒）");
        
        let clients = self.clients.clone();
        
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    info!("Napcat 已连接: {}", addr);
                    let clients_clone = clients.clone();
                    
                    tokio::spawn(async move {
                        match accept_async(stream).await {
                            Ok(ws_stream) => {
                                info!("WebSocket 握手成功: {}", addr);
                                clients_clone.lock().await.push(ws_stream);
                            }
                            Err(e) => {
                                error!("WebSocket 握手失败: {}", e);
                            }
                        }
                    });
                }
                Err(e) => {
                    error!("接受连接失败: {}", e);
                }
            }
        }
    }
    
    /// 接收一条消息
    pub async fn receive_one(&self) -> Result<Option<Value>> {
        let mut clients = self.clients.lock().await;
        
        for i in (0..clients.len()).rev() {
            let client = &mut clients[i];
            
            match client.next().await {
                Some(Ok(Message::Text(text))) => {
                    let data: Value = serde_json::from_str(&text)?;
                    return Ok(Some(data));
                }
                Some(Ok(Message::Binary(bin))) => {
                    let data: Value = serde_json::from_slice(&bin)?;
                    return Ok(Some(data));
                }
                Some(Err(e)) => {
                    error!("接收消息错误: {}", e);
                    clients.remove(i);
                }
                None => {
                    // 连接关闭
                    clients.remove(i);
                }
                _ => {}
            }
        }
        
        Ok(None)
    }
}