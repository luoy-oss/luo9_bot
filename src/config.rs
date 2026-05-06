use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use crate::error::Result;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LNConfig {
    pub napcat: NapcatConfig,
    pub logging: LoggingConfig,
    #[serde(default)]
    pub plugins: PluginConfig,
    #[serde(default)]
    pub webui: WebuiConfig,
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebuiConfig {
    #[serde(default = "default_webui_enabled")]
    pub enabled: bool,
    #[serde(default = "default_webui_host")]
    pub host: String,
    #[serde(default = "default_webui_port")]
    pub port: u16,
    /// WebUI 访问 token，为空时每次启动随机生成
    #[serde(default)]
    pub token: String,
}

impl Default for WebuiConfig {
    fn default() -> Self {
        Self {
            enabled: default_webui_enabled(),
            host: default_webui_host(),
            port: default_webui_port(),
            token: String::new(),
        }
    }
}

fn default_webui_enabled() -> bool {
    true
}

fn default_webui_host() -> String {
    "0.0.0.0".to_string()
}

fn default_webui_port() -> u16 {
    27080
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
    /// 解析配置文件路径，优先级：
    /// 1. 环境变量 LUO9_CONFIG
    /// 2. ~/.luo9/config/default.toml
    /// 3. config/default.toml（向后兼容）
    pub fn config_path() -> PathBuf {
        if let Ok(path) = env::var("LUO9_CONFIG") {
            return PathBuf::from(path);
        }

        if let Some(home) = dirs::home_dir() {
            let luo9_path = home.join(".luo9").join("config").join("default.toml");
            if luo9_path.exists() {
                return luo9_path;
            }
        }

        PathBuf::from("config/default.toml")
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        let content = fs::read_to_string(&path)?;

        let mut config: LNConfig = toml::from_str(&content)?;

        // 环境变量覆盖插件目录
        if let Ok(plugin_dir) = env::var("LUO9_PLUGIN_DIR") {
            config.plugins.plugin_dir = plugin_dir;
        }

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