use thiserror::Error;

#[derive(Debug, Error)]
pub enum LNErr {
    #[error("配置错误: {0}")]
    Config(String),
    
    #[error("WebSocket 错误: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
    
    #[error("JSON 错误: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("TOML 解析错误: {0}")]
    Toml(#[from] toml::de::Error),
    
    #[error("JSON 解析错误: {0}")]
    JsonParseError(String),
    
    #[error("事件解析错误: {0}")]
    EventParseError(String),

    #[error("无效的 HTTP 头值: {0}")]
    InvalidHeaderValue(String),
    
    #[error("未知的消息类型")]
    UnknownMsgType,
    
    #[error("无效的消息格式")]
    InvalidMessage,
    
    #[error("未知的元事件类型")]
    UnknownMetaEventType,
    
    #[error("未知的通知类型")]
    UnknownNoticeType,
    
}

pub type Result<T> = std::result::Result<T, LNErr>;
