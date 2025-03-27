//! Sky 插件
//! 
//! 提供光遇API，包括红石查询、每日任务查询等功能

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::{Result, anyhow};
use luo9_bot::config::Value;
use luo9_bot::core::plugin_manager::{Plugin, PluginMetadata};
use luo9_bot::core::message::GroupMessage;
use luo9_bot::export_plugin;
use luo9_bot::api::ApiManager;
use luo9_bot::utils::message_limit::MessageLimit;
use luo9_bot::utils::download_img;

/// Sky 插件结构
pub struct SkyPlugin {
    metadata: PluginMetadata,
    config: Arc<Value>,
    api: ApiManager,
    skyhs_limit: MessageLimit,
    skyrw_limit: MessageLimit,
    skyjl_limit: MessageLimit,
}

impl SkyPlugin {
    /// 创建一个新的 Sky 插件
    pub async fn new(config: Arc<Value>) -> Result<Self> {
        let metadata = PluginMetadata {
            name: "sky".to_string(),
            describe: "光遇api，提供红石查询 每日任务查询".to_string(),
            author: "drluo".to_string(),
            version: "1.0.0".to_string(),
            message_types: vec![
                "group_message".to_string(),
            ],
        };
        
        // 初始化 API
        let api = match ApiManager::new(config.clone()) {
            Ok(api) => api,
            Err(e) => return Err(anyhow!("API初始化失败: {}", e)),
        };
        
        Ok(Self {
            metadata,
            config,
            api,
            skyhs_limit: MessageLimit::new("skyhs"),
            skyrw_limit: MessageLimit::new("skyrw"),
            skyjl_limit: MessageLimit::new("skyjl"),
        })
    }
}

#[async_trait]
impl Plugin for SkyPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    async fn handle_group_message(&self, message: &GroupMessage) -> Result<()> {
        let group_id = &message.group_id;
        let content = &message.content;
        let data_path = &self.config.data_path;
        
        // 创建可变引用以便修改限制器状态
        let mut skyhs_limit = self.skyhs_limit.clone();
        let mut skyrw_limit = self.skyrw_limit.clone();
        let mut skyjl_limit = self.skyjl_limit.clone();
        
        // 处理红石查询
        if skyhs_limit.check(30.0) && (content == "sky红石" || content == "sky红石雨" || content == "sky黑石") {
            skyhs_limit.handle();
            let img_url = "https://api.zxz.ee/api/sky/?type=&lx=hs";
            let save_path = format!("{}/plugins/{}/hs.jpg", data_path, self.metadata.name);
            
            if let Err(e) = download_img::download_image_if_needed(content, img_url, &save_path).await {
                println!("下载图片失败: {}", e);
                return Ok(());
            }
            println!("图片已下载: {}", save_path);
            if let Err(e) = self.api.send_group_image(group_id, &save_path).await {
                println!("发送图片失败: {}", e);
            }
        }
        
        // 处理每日任务查询
        if skyrw_limit.check(30.0) && (content == "sky每日任务" || content == "sky每日" || content == "sky任务") {
            skyrw_limit.handle();
            let img_url = "https://api.zxz.ee/api/sky/?type=&lx=rw";
            let save_path = format!("{}/plugins/{}/rw.jpg", data_path, self.metadata.name);
            
            if let Err(e) = download_img::download_image_if_needed(content, img_url, &save_path).await {
                println!("下载图片失败: {}", e);
                return Ok(());
            }
            
            if let Err(e) = self.api.send_group_image(group_id, &save_path).await {
                println!("发送图片失败: {}", e);
            }
        }
        
        // 处理季节蜡烛查询
        if skyjl_limit.check(30.0) && (content == "sky季节蜡烛" || content == "sky季蜡") {
            skyjl_limit.handle();
            let img_url = "https://api.zxz.ee/api/sky/?type=&lx=jl";
            let save_path = format!("{}/plugins/{}/jl.jpg", data_path, self.metadata.name);
            
            if let Err(e) = download_img::download_image_if_needed(content, img_url, &save_path).await {
                println!("下载图片失败: {}", e);
                return Ok(());
            }
            
            if let Err(e) = self.api.send_group_image(group_id, &save_path).await {
                println!("发送图片失败: {}", e);
            }
        }
        
        Ok(())
    }
}

// 创建插件实例的异步函数
async fn create(config: Arc<Value>) -> Result<Box<dyn Plugin>> {
    let plugin = SkyPlugin::new(config).await?;
    Ok(Box::new(plugin))
}

// 导出插件创建函数
export_plugin!(create);