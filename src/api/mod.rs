pub mod napcat;

use std::sync::Arc;
use crate::config::Value;

#[async_trait::async_trait]
pub trait ApiTrait: Send + Sync {
    async fn send_group_message(&self, group_id: &str, message: &str) -> Result<(), Box<dyn std::error::Error>>;
    async fn send_private_msg(&self, user_id: &str, message: &str) -> Result<(), Box<dyn std::error::Error>>;
    async fn send_group_ai_record(&self, group_id: &str, voice: &str, message: &str) -> Result<(), Box<dyn std::error::Error>>;
    async fn send_group_at(&self, group_id: &str, qq: &str) -> Result<(), Box<dyn std::error::Error>>;
    async fn send_group_image(&self, group_id: &str, file: &str) -> Result<(), Box<dyn std::error::Error>>;
    async fn send_group_file(&self, group_id: &str, file: &str, name: &str, folder_id: &str) -> Result<(), Box<dyn std::error::Error>>;
    async fn send_group_poke(&self, group_id: &str, user_id: &str) -> Result<(), Box<dyn std::error::Error>>;
    async fn get_group_files_by_folder(&self, group_id: &str, folder_id: &str, file_count: i32) -> Result<String, Box<dyn std::error::Error>>;
    async fn get_group_root_files(&self, group_id: &str) -> Result<String, Box<dyn std::error::Error>>;
}

#[derive(Clone)]
pub struct ApiManager {
    api: Arc<dyn ApiTrait>
}

impl ApiManager {
    pub fn new(config: Arc<Value>) -> Result<Self, Box<dyn std::error::Error>> {
        let _config  = config.clone();
        
        if _config.napcat {
            let napcat = napcat::NapCat::new(_config.base_url(), _config.access_token());
            return Ok(Self {
                api: Arc::new(napcat)
            });
        }
     
        // 如果需要支持QQBot，可以在这里添加
        // if let Some(qqbot_config) = &config.qqbot {
        //     if qqbot_config.enable {
        //         let qqbot = qqbot::QQBot::new(config.base_url, config.access_token);
        //         return Ok(Self {
        //             api: Arc::new(qqbot),
        //         });
        //     }
        // }
        
        Err("No API enabled in config".into())
    }
    
    // 群聊消息
    pub async fn send_group_message(&self, group_id: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.api.send_group_message(group_id, message).await
    }

    // 群聊AI语音
    pub async fn send_group_ai_record(&self, group_id: &str, voice: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.api.send_group_ai_record(group_id, voice, message).await
    }
    
    // 群聊AT
    pub async fn send_group_at(&self, group_id: &str, qq: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.api.send_group_at(group_id, qq).await
    }
    
    // 群聊图片
    pub async fn send_group_image(&self, group_id: &str, file: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.api.send_group_image(group_id, file).await
    }
    
    // 群聊文件
    pub async fn send_group_file(&self, group_id: &str, file: &str, name: &str, folder_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.api.send_group_file(group_id, file, name, folder_id).await
    }
    
    // 群聊戳一戳
    pub async fn send_group_poke(&self, group_id: &str, user_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.api.send_group_poke(group_id, user_id).await
    }
    
    // 私聊消息
    pub async fn send_private_msg(&self, user_id: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.api.send_private_msg(user_id, message).await
    }
    
    // 获取群文件
    pub async fn get_group_files_by_folder(&self, group_id: &str, folder_id: &str, file_count: i32) -> Result<String, Box<dyn std::error::Error>> {
        self.api.get_group_files_by_folder(group_id, folder_id, file_count).await
    }
    
    // 获取群根目录文件
    pub async fn get_group_root_files(&self, group_id: &str) -> Result<String, Box<dyn std::error::Error>> {
        self.api.get_group_root_files(group_id).await
    }
}