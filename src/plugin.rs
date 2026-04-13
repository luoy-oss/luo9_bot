use libloading::{Library, Symbol};
use std::ffi::CString;
use std::sync::Mutex;

use crate::config::PluginConfig;

use tracing::{info, warn, error};

// 全局插件列表（静态 + 线程安全）
static PLUGINS: Mutex<Vec<Library>> = Mutex::new(Vec::new());

/// 加载插件目录下所有 .dll/.so 文件
pub fn load_all_plugins(config: &PluginConfig) {
    let mut plugins = PLUGINS.lock().unwrap();
    plugins.clear();

    // 遍历 plugins 目录加载所有插件
    if !std::path::Path::new(&config.plugin_dir).exists() {
        warn!("插件目录不存在 将不进行插件载入: {}", config.plugin_dir);
        return;
    }
    let paths = std::fs::read_dir(&config.plugin_dir).unwrap();
    info!("插件目录: {}", config.plugin_dir);
    for entry in paths {
        let entry = entry.unwrap();
        let path = entry.path();
        
        // 只加载动态库
        if path.extension().and_then(|s| s.to_str()) == Some("dll") || 
           path.extension().and_then(|s| s.to_str()) == Some("so") 
        {
            unsafe {
                let lib = Library::new(path).unwrap_or_else(|e| {
                    error!("加载插件失败: {}", e);
                    std::process::exit(1);
                });
                plugins.push(lib);
                info!("✅ 插件加载成功: {}", entry.path().display());
            }
        }
    }
}


pub fn luo9_cleanup() {
    let mut plugins = PLUGINS.lock().unwrap();
    plugins.clear();
    println!("[Luo9] 已清理所有插件");
}

/// 分发私聊消息
pub fn dispatch_pmsg(user_id: u64, json: &str) {
    let plugins = PLUGINS.lock().unwrap();
    
    let c_json = CString::new(json).unwrap();
    let c_ptr = c_json.as_ptr();

    for lib in &*plugins {
        unsafe {
            let func: Symbol<extern "C" fn(u64, *const i8) -> ()> = lib
                .get(b"pmsg_process\0")
                .unwrap();

            func(user_id, c_ptr); // 调用插件
        }
    }
}

/// 分发群聊消息
pub fn dispatch_gmsg(group_id: u64, user_id: u64, json: &str) {
    let plugins = PLUGINS.lock().unwrap();
    
    let c_json = CString::new(json).unwrap();
    let c_ptr = c_json.as_ptr();

    for lib in &*plugins {
        unsafe {
            #[cfg(feature = "plugin_dispatch_debug")]
            info!("分发群聊消息: {:?}", json);

            let func: Symbol<extern "C" fn(u64, u64, *const i8) -> ()> = lib
                .get(b"gmsg_process\0")
                .unwrap();

            func(group_id, user_id, c_ptr); // 调用插件
        }
    }
}

/// 分发私聊事件
pub fn dispatch_pevent(_msg: &str) {

}

/// 分发群聊事件
pub fn dispatch_gevent(_msg: &str) {
}

