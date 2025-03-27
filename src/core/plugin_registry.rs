//! 插件注册系统
//! 
//! 这个模块提供了一个插件注册表，用于管理所有可用的插件。

use std::collections::HashMap;
use std::sync::Arc;
use std::path::Path;
use anyhow::{Result, anyhow};
use lazy_static::lazy_static;
use std::sync::Mutex as StdMutex;
use libloading::{Library, Symbol};

use crate::config::Value;
use super::plugin_manager::Plugin;

/// 插件工厂函数类型
type PluginFactory = fn(Arc<Value>) -> Result<Box<dyn Plugin>>;

/// 插件注册表
pub struct PluginRegistry {
    factories: HashMap<String, PluginFactory>,
    libraries: HashMap<String, Library>, // 保存已加载的动态库
}

impl PluginRegistry {
    /// 创建一个新的插件注册表
    pub fn new() -> Self {
        let mut registry = Self {
            factories: HashMap::new(),
            libraries: HashMap::new(),
        };
        
        // 注册内置插件
        registry.register_builtin_plugins();
        
        registry
    }
    
    /// 注册内置插件
    fn register_builtin_plugins(&mut self) {
        // 在这里注册内置插件
        // 例如：
        // self.register("example_plugin", |config| {
        //     Ok(Box::new(ExamplePlugin::new(config)?))
        // });
    }
    
    /// 注册一个插件
    pub fn register(&mut self, name: &str, factory: PluginFactory) {
        self.factories.insert(name.to_string(), factory);
    }
    
    /// 加载外部插件
    pub fn load_external_plugin(&mut self, plugin_dir: &Path, name: &str) -> Result<()> {
        let plugin_path = plugin_dir.join(name);
        
        // 检查插件目录是否存在
        if !plugin_path.is_dir() {
            return Err(anyhow!("插件目录不存在: {}", plugin_path.display()));
        }
        
        // 构建动态库路径
        #[cfg(target_os = "windows")]
        let lib_path = plugin_path.join("target/release").join(format!("{}.dll", name));
        
        #[cfg(target_os = "linux")]
        let lib_path = plugin_path.join("target/release").join(format!("lib{}.so", name));
        
        #[cfg(target_os = "macos")]
        let lib_path = plugin_path.join("target/release").join(format!("lib{}.dylib", name));
        
        // 检查动态库是否存在
        if !lib_path.exists() {
            return Err(anyhow!("插件动态库不存在: {}", lib_path.display()));
        }
        
        // 加载动态库
        unsafe {
            let lib = Library::new(&lib_path)?;
            
            // 获取插件创建函数
            let create_plugin: Symbol<fn(Arc<Value>) -> Result<Box<dyn Plugin>>> = 
                lib.get(b"create_plugin")?;
            
            // 注册插件工厂
            let factory = *create_plugin;
            self.register(name, factory);
            
            // 保存库引用，防止被提前释放
            self.libraries.insert(name.to_string(), lib);
            
            Ok(())
        }
    }
    
    /// 创建一个插件实例
    pub fn create(&self, name: &str, config: Arc<Value>) -> Result<Box<dyn Plugin>> {
        match self.factories.get(name) {
            Some(factory) => factory(config),
            None => Err(anyhow!("未知的插件: {}", name)),
        }
    }
}

// 创建一个全局的插件注册表
lazy_static! {
    pub static ref PLUGIN_REGISTRY: StdMutex<PluginRegistry> = StdMutex::new(PluginRegistry::new());
}

// 注册插件的宏
#[macro_export]
macro_rules! register_plugin {
    ($name:expr, $factory:expr) => {
        #[cfg(feature = "plugin_registry")]
        #[ctor::ctor]
        fn register_plugin() {
            let mut registry = $crate::core::plugin_registry::PLUGIN_REGISTRY.lock().unwrap();
            registry.register($name, $factory);
        }
    };
}

// 导出插件创建函数的宏
#[macro_export]
macro_rules! export_plugin {
    ($create_fn:expr) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn create_plugin(
            config: std::sync::Arc<luo9_bot::config::Value>,
        ) -> anyhow::Result<Box<dyn luo9_bot::core::plugin_manager::Plugin>> {
            // 检查是否在 Tokio 运行时上下文中
            match tokio::runtime::Handle::try_current() {
                // 如果在 Tokio 运行时上下文中，直接使用当前运行时
                Ok(handle) => handle.block_on($create_fn(config)),
                // 如果不在 Tokio 运行时上下文中，创建一个新的运行时
                Err(_) => {
                    // 创建一个新的 Tokio 运行时
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .expect("无法创建 Tokio 运行时");
                    
                    // 在新创建的运行时中执行异步函数
                    rt.block_on($create_fn(config))
                }
            }
        }
    };
}