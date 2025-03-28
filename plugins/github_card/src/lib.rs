use std::sync::Arc;
use std::collections::HashMap;
use async_trait::async_trait;
use anyhow::{Result, anyhow};
use luo9_bot::config::Value;
use luo9_bot::core::plugin_manager::{Plugin, PluginMetadata};
use luo9_bot::core::message::GroupMessage;
use luo9_bot::export_plugin;
use luo9_bot::api::ApiManager;
use luo9_bot::utils::message_limit::MessageLimit;
use luo9_bot::utils::download_img;
use regex::Regex;

/// GitHub Card 插件结构
pub struct GitHubCardPlugin {
    metadata: PluginMetadata,
    config: Arc<Value>,
    api: ApiManager,
    github_card_limit: MessageLimit,
}

impl GitHubCardPlugin {
    /// 创建一个新的 GitHub Card 插件
    pub async fn new(config: Arc<Value>) -> Result<Self> {
        let metadata = PluginMetadata {
            name: "github_card".to_string(),
            describe: "github链接解析为图片".to_string(),
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
            github_card_limit: MessageLimit::new("github_card"),
        })
    }
    
    /// 获取GitHub仓库信息
    async fn get_github_repository_information(&self, url: &str) -> Result<HashMap<String, String>> {
        // 移除协议前缀
        let url = url.trim_start_matches("http://").trim_start_matches("https://");
        let url = url.trim_start_matches("github.com/");
        
        let parts: Vec<&str> = url.split('/').collect();
        
        if parts.len() < 2 {
            return Err(anyhow!("无效的GitHub URL"));
        }
        
        let user_name = parts[0];
        let repo_name = parts[1];
        let image_url = format!("https://opengraph.githubassets.com/githubcard/{}/{}", user_name, repo_name);
        
        let mut result = HashMap::new();
        result.insert("image_url".to_string(), image_url);
        result.insert("user_name".to_string(), user_name.to_string());
        result.insert("repo_name".to_string(), repo_name.to_string());
        
        Ok(result)
    }
}

#[async_trait]
impl Plugin for GitHubCardPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    async fn handle_group_message(&self, message: &GroupMessage) -> Result<()> {
        let group_id = &message.group_id;
        let content = &message.content;
        let data_path = &self.config.data_path;
        
        // 创建可变引用以便修改限制器状态
        let mut github_card_limit = self.github_card_limit.clone();
        
        if content.contains("github.com") && github_card_limit.check(10.0) {
            github_card_limit.handle();
            
            // 使用正则表达式提取GitHub URL
            let re = Regex::new(r"(https?://)?github\.com/[^\s]+").unwrap();
            if let Some(captures) = re.find(content) {
                let github_url = captures.as_str();
                
                match self.get_github_repository_information(github_url).await {
                    Ok(github_card) => {
                        let save_path = format!(
                            "{}/plugins/{}/{}.{}.jpg", 
                            data_path, 
                            self.metadata.name, 
                            github_card["user_name"], 
                            github_card["repo_name"]
                        );
                        
                        if let Err(e) = download_img::download_image_if_needed(
                            content, 
                            &github_card["image_url"], 
                            &save_path
                        ).await {
                            println!("下载GitHub卡片图片失败: {}", e);
                            return Ok(());
                        }

                        match self.api.send_group_image(group_id, &save_path).await {
                            Ok(_) => (),
                            Err(e) => println!("发送GitHub卡片图片失败: {}", e),
                        }
                    },
                    Err(e) => {
                        println!("解析GitHub URL失败: {}", e);
                    }
                }
            }
        }
        
        Ok(())
    }
}

// 创建插件实例的异步函数
async fn create(config: Arc<Value>) -> Result<Box<dyn Plugin>> {
    let plugin = GitHubCardPlugin::new(config).await?;
    Ok(Box::new(plugin))
}

// 导出插件创建函数
export_plugin!(create);