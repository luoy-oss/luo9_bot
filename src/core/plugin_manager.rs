//! 插件管理器模块
//! 
//! 这个模块提供了一个泛型的插件系统，用于加载和管理各种插件。

use std::fs;
use std::path::Path;
use std::sync::Arc;
use anyhow::{Result, anyhow};
use serde::Deserialize;
use serde_yaml;

// 使用新的SDK
use luo9_sdk::plugin::Plugin;
use luo9_sdk::message::{GroupMessage, PrivateMessage};
use luo9_sdk::config::Value;

/// 插件配置结构
#[derive(Debug, Deserialize)]
struct PluginConfig {
    plugins: Vec<PluginEntry>,
}

/// 插件条目结构
#[derive(Debug, Deserialize)]
struct PluginEntry {
    name: String,
    enable: bool,
    priority: i32,
}

/// 插件信息结构
struct PluginInfo {
    plugin: Arc<Box<dyn Plugin>>,
    priority: i32,
}

/// 插件管理器
pub struct PluginManager {
    plugins: Vec<PluginInfo>,
    config: Arc<Value>,
}

impl PluginManager {
    /// 创建一个新的插件管理器
    pub async fn new(config: Arc<Value>) -> Result<Self> {
        let mut manager = Self {
            plugins: Vec::new(),
            config: config.clone(),
        };
        
        manager.load_plugins().await?;
        
        Ok(manager)
    }
    
    /// 加载插件
    async fn load_plugins(&mut self) -> Result<()> {
        let plugin_dir = &self.config.plugin_path;
        let config_path = Path::new(plugin_dir).join("config.yaml");
        tracing::info!("插件配置文件路径：{}", config_path.display());
        
        let config_content = fs::read_to_string(&config_path)
            .map_err(|e| anyhow!("无法读取插件配置文件: {}", e))?;
        
        let config: PluginConfig = serde_yaml::from_str(&config_content)
            .map_err(|e| anyhow!("解析插件配置文件失败: {}", e))?;

        tracing::info!("插件总数：{}", config.plugins.len());        
        tracing::info!("---------------------------");();
        
        let mut load_num = 0;
        
        for plugin_entry in &config.plugins {
            if plugin_entry.enable {
                let plugin_name = &plugin_entry.name;
                let plugin_path = Path::new(plugin_dir).join(plugin_name);
                
                if plugin_path.is_dir() {
                    // 创建插件数据目录
                    let plugin_data_path = Path::new(&self.config.data_path).join("plugins").join(plugin_name);
                    tracing::info!("插件数据目录：{}", plugin_data_path.display());
                    
                    if !plugin_data_path.exists() {
                        fs::create_dir_all(&plugin_data_path)
                            .map_err(|e| anyhow!("创建插件数据目录失败: {}", e))?;
                        
                        #[cfg(not(target_os = "windows"))]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            let metadata = fs::metadata(&plugin_data_path)?;
                            let mut permissions = metadata.permissions();
                            permissions.set_mode(0o777);
                            fs::set_permissions(&plugin_data_path, permissions)?;
                        }
                    }
                    
                    // 加载插件
                    match self.load_plugin(plugin_name, plugin_entry.priority).await {
                        Ok(plugin) => {
                            let metadata = plugin.metadata();                            
                            tracing::info!(
                                "\"插件\'：{}, \"作者\"：{}, \"插件描述\"：{}, \"版本\"：{}, \"插件需求\"：{:?}",
                                metadata.name, metadata.author, metadata.describe, metadata.version, metadata.message_types
                            );
                            tracing::info!("---------------------------");();
                            // 在 load_plugins 方法中
                            self.plugins.push(PluginInfo {
                                plugin: Arc::new(plugin),  // 使用 Arc 包装 Box<dyn Plugin>
                                priority: plugin_entry.priority,
                            });
                            
                            load_num += 1;
                        },
                        Err(e) => {
                            tracing::info!("加载插件 {} 失败: {}", plugin_name, e);
                            tracing::info!("---------------------------");();
                        }
                    }
                }
            }
        }
        
        tracing::info!("加载完成：{}/{}", load_num, config.plugins.len());
        tracing::info!("---------------------------");();
        
        Ok(())
    }
    
    /// 加载单个插件
    async fn load_plugin(&self, name: &str, _priority: i32) -> Result<Box<dyn Plugin>> {
        // 使用libloading直接加载插件DLL
        let plugin_dir = Path::new(&self.config.plugin_path);
        let plugin_path = plugin_dir.join(name);
        
        // 构建动态库路径
        #[cfg(target_os = "windows")]
        let lib_path = plugin_path.join(format!("{}.dll", name));
        
        #[cfg(target_os = "linux")]
        let lib_path = plugin_path.join(format!("lib{}.so", name));
        
        #[cfg(target_os = "macos")]
        let lib_path = plugin_path.join(format!("lib{}.dylib", name));
        
        // 检查动态库是否存在
        if !lib_path.exists() {
            return Err(anyhow!("插件动态库不存在: {}", lib_path.display()));
        }
        
        // 在单独的线程中执行插件加载操作
        let lib_path_clone = lib_path.to_path_buf();
        let config_clone = self.config.clone();
        
        let plugin = tokio::task::spawn_blocking(move || {
            unsafe {
                // 加载动态库
                let lib = libloading::Library::new(&lib_path_clone)?;
                
                // 获取插件创建函数
                let create_plugin: libloading::Symbol<fn(Arc<Value>) -> Result<Box<dyn Plugin>>> = 
                    lib.get(b"create_plugin")?;
                
                // 创建插件实例
                let plugin = create_plugin(config_clone)?;
                
                // 注意：这里有内存泄漏风险，因为我们没有保存lib的引用
                // 在实际应用中，应该将lib保存在某个地方，防止被提前释放
                std::mem::forget(lib);
                
                Ok::<Box<dyn Plugin>, anyhow::Error>(plugin)
            }
        }).await??;
        
        Ok(plugin)
    }
    
    /// 处理群消息
    pub async fn handle_group_message(&self, message: &GroupMessage) -> Result<()> {
        // 按优先级排序插件
        let mut plugins = self.plugins.iter().collect::<Vec<_>>();
        plugins.sort_by_key(|p| p.priority);
        
        // 创建一个任务集合
        let mut tasks = Vec::new();
        
        for plugin_info in plugins {
            let metadata = plugin_info.plugin.metadata();
            if metadata.message_types.contains(&"group_message".to_string()) {
                // 使用 Arc 克隆而不是直接克隆
                let plugin = plugin_info.plugin.clone();
                let message_clone = message.clone();
                
                // 创建一个新任务
                let task = tokio::spawn(async move {
                    match plugin.handle_group_message(&message_clone).await {
                        Ok(_) => (),
                        Err(e) => println!("插件 {} 处理群消息失败: {}", plugin.metadata().name, e),
                    }
                });
                
                tasks.push(task);
            }
        }
        
        
        Ok(())
    }
    
    /// 处理私聊消息
    pub async fn handle_private_message(&self, message: &PrivateMessage) -> Result<()> {
        // 按优先级排序插件
        let mut plugins = self.plugins.iter().collect::<Vec<_>>();
        plugins.sort_by_key(|p| p.priority);
        
        // 创建一个任务集合
        let mut tasks = Vec::new();
        
        for plugin_info in plugins {
            let metadata = plugin_info.plugin.metadata();
            if metadata.message_types.contains(&"private_message".to_string()) {
                // 使用 Arc 克隆而不是直接克隆
                let plugin = plugin_info.plugin.clone();
                let message_clone = message.clone();
                
                // 创建一个新任务
                let task = tokio::spawn(async move {
                    match plugin.handle_private_message(&message_clone).await {
                        Ok(_) => (),
                        Err(e) => println!("插件 {} 处理私聊消息失败: {}", plugin.metadata().name, e),
                    }
                });
                
                tasks.push(task);
            }
        }
        
        Ok(())
    }
    
    /// 处理群戳一戳事件
    pub async fn handle_group_poke(&self, target_id: &str, user_id: &str, group_id: &str) -> Result<()> {
        // 按优先级排序插件
        let mut plugins = self.plugins.iter().collect::<Vec<_>>();
        plugins.sort_by_key(|p| p.priority);
        
        // 创建一个任务集合
        let mut tasks = Vec::new();
        
        for plugin_info in plugins {
            let metadata = plugin_info.plugin.metadata();
            if metadata.message_types.contains(&"group_poke".to_string()) {
                // 使用 Arc 克隆而不是直接克隆
                let plugin = plugin_info.plugin.clone();
                let target_id = target_id.to_string();
                let user_id = user_id.to_string();
                let group_id = group_id.to_string();
                
                // 创建一个新任务
                let task = tokio::spawn(async move {
                    match plugin.handle_group_poke(&target_id, &user_id, &group_id).await {
                        Ok(_) => (),
                        Err(e) => println!("插件 {} 处理群戳一戳事件失败: {}", plugin.metadata().name, e),
                    }
                });
                
                tasks.push(task);
            }
        }
        Ok(())
    }
}
