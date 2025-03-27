//! 配置值模块
//! 
//! 提供配置值的访问接口，类似于Python版本中的Value类。

use std::path::PathBuf;
use super::Config;

/// 配置值结构体
#[derive(Debug, Clone)]
pub struct Value {
    /// 项目路径
    pub path: String,
    /// 数据路径
    pub data_path: String,
    /// 插件路径
    pub plugin_path: String,
    /// 核心路径
    pub core_path: String,
    /// 机器人ID
    pub bot_id: u64,
    /// 主人ID
    pub master: u64,
    /// 群组列表
    pub group_list: Vec<u64>,
    /// B站直播检测推送列表
    pub bilibili_live_push_list: Vec<u64>,
    /// 节日检测推送列表
    pub festival_push_list: Vec<u64>,
    /// 土豆直播间ID
    pub tudou_live_id: u64,
    /// AI语音音色
    pub ai_voice_type: String,
    /// 是否启用NapCat
    pub napcat: bool,
    /// NapCat服务器主机
    pub ncs_host: String,
    /// NapCat服务器端口
    pub ncs_port: u16,
    /// NapCat服务器令牌
    pub ncs_token: String,
    /// NapCat客户端主机
    pub ncc_host: String,
    /// NapCat客户端端口
    pub ncc_port: u16,
    /// NapCat客户端令牌
    pub ncc_token: String,
}

impl Value {
    /// 从配置创建Value实例
    pub fn new(config: &Config) -> Self {
        let path = config.path.clone();
        let data_path = format!("{}/data", path);
        let plugin_path = format!("{}/plugins", path);
        let core_path = format!("{}/luo9", path);
        
        let mut value = Self {
            path,
            data_path,
            plugin_path,
            core_path,
            bot_id: config.bot_id,
            master: config.master,
            group_list: config.group_list.clone(),
            bilibili_live_push_list: config.bilibili_live_push_list.clone(),
            festival_push_list: config.festival_push_list.clone(),
            tudou_live_id: config.tudou_live_id,
            ai_voice_type: config.ai_voice_type.clone(),
            napcat: config.napcat.enable,
            ncs_host: String::new(),
            ncs_port: 0,
            ncs_token: String::new(),
            ncc_host: String::new(),
            ncc_port: 0,
            ncc_token: String::new(),
        };
        
        if value.napcat {
            value.ncs_host = config.napcat.http_servers.host.clone();
            value.ncs_port = config.napcat.http_servers.port;
            value.ncs_token = config.napcat.http_servers.token.clone();
            
            value.ncc_host = config.napcat.http_clients.host.clone();
            value.ncc_port = config.napcat.http_clients.port;
            value.ncc_token = config.napcat.http_clients.token.clone();
        }
        
        value
    }
    
    /// 获取机器人消息推送基础URL
    pub fn base_url(&self) -> String {
        if self.napcat {
            format!("{}:{}", self.ncs_host, self.ncs_port)
        } else {
            String::new()
        }
    }
    
    /// 获取机器人访问令牌
    pub fn access_token(&self) -> String {
        if self.napcat {
            self.ncs_token.clone()
        } else {
            String::new()
        }
    }
    
    /// 获取数据目录的PathBuf
    pub fn data_path_buf(&self) -> PathBuf {
        PathBuf::from(&self.data_path)
    }
    
    /// 获取插件目录的PathBuf
    pub fn plugin_path_buf(&self) -> PathBuf {
        PathBuf::from(&self.plugin_path)
    }
}