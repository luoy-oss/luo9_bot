use serde::{Deserialize, Serialize};
use std::fs;
use crate::error::Result;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LNConfig {
    pub napcat: NapcatConfig,
    pub logging: LoggingConfig,
    #[serde(default)]
    pub plugins: PluginConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NapcatConfig {
    pub ws_client_host: String,
    pub ws_client_port: u16,
    pub ws_server_host: String,
    pub ws_server_port: u16,
    pub timeout_seconds: u64,
    pub token: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub level: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PluginConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_plugin_dir")]
    pub plugin_dir: String,
    #[serde(default = "default_auto_load")]
    pub auto_load: bool,
    #[serde(default)]
    pub plugins: Vec<String>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            plugin_dir: default_plugin_dir(),
            auto_load: default_auto_load(),
            plugins: Vec::new(),
        }
    }
}

fn default_enabled() -> bool {
    false
}

fn default_plugin_dir() -> String {
    "plugins".to_string()
}

fn default_auto_load() -> bool {
    true
}

impl LNConfig {
    pub fn load() -> Result<Self> {
        let content = fs::read_to_string("config/default.toml")?;
        
        let config: LNConfig = toml::from_str(&content)?;
        println!("{:?}", config);
        
        Ok(config)
    }
    
    // 修复 receiver_url 和 sender_url 方法
    pub fn receiver_url(&self) -> String {
        format!("ws://{}:{}", self.napcat.ws_client_host, self.napcat.ws_client_port)
    }
    
    pub fn sender_url(&self) -> String {
        format!("ws://{}:{}", self.napcat.ws_server_host, self.napcat.ws_server_port)
    }
}