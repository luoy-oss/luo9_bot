//! 配置模块
//! 
//! 这个模块负责加载和管理机器人的配置信息。

mod value;

pub use value::Value;

use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

/// 机器人配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// NapCat配置
    pub napcat: NapCatConfig,
    /// 项目路径
    #[serde(rename = "PATH")]
    pub path: String,
    /// 机器人ID
    pub bot_id: u64,
    /// 主人ID
    pub master: u64,
    /// 群组列表
    pub group_list: Vec<u64>,
    /// B站直播检测推送列表
    #[serde(rename = "B站直播检测推送列表")]
    pub bilibili_live_push_list: Vec<u64>,
    /// 节日检测推送列表
    #[serde(rename = "节日检测推送列表")]
    pub festival_push_list: Vec<u64>,
    /// 土豆直播间ID
    #[serde(rename = "土豆直播间ID")]
    pub tudou_live_id: u64,
    /// AI语音音色
    #[serde(rename = "AI语音音色")]
    pub ai_voice_type: String,
}

/// NapCat配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NapCatConfig {
    /// 是否启用NapCat
    pub enable: bool,
    /// HTTP服务器配置
    #[serde(rename = "httpServers")]
    pub http_servers: HttpConfig,
    /// HTTP客户端配置
    #[serde(rename = "httpClients")]
    pub http_clients: HttpConfig,
}

/// HTTP配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    /// 是否启用
    pub enable: bool,
    /// 主机地址
    pub host: String,
    /// 端口
    pub port: u16,
    /// 令牌
    pub token: String,
}

/// 加载配置文件
pub fn load_config(file_path: &str) -> Result<Config> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(anyhow::anyhow!(
            "请检查config.(example).yaml同级目录下的配置文件 {} 是否创建",
            file_path
        ));
    }
    
    let config_str: String = fs::read_to_string(path)
        .with_context(|| format!("无法读取配置文件: {}", file_path))?;
    
    println!("{}", config_str);

    let config: Config = serde_yaml::from_str(&config_str)
        .with_context(|| "解析配置文件失败")?;
    
    tracing::info!("配置加载成功");
    Ok(config)
}
