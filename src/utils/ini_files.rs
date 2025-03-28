use anyhow::{anyhow, Result};
use configparser::ini::Ini;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub async fn read_config_item(
    file: &str,
    section_name: &str,
    config_item_name: &str,
    default_value: &str,
) -> Result<String> {
    if !Path::new(file).exists() {
        let mut file_handle = File::create(file)?;
        file_handle.write_all(b"")?;
        
        #[cfg(not(target_os = "windows"))]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(file)?;
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o777);
            fs::set_permissions(file, permissions)?;
        }
    }

    let mut config = Ini::new();
    // 处理 load 方法可能返回的错误
    if let Err(e) = config.load(file) {
        return Err(anyhow!("加载配置文件失败: {}", e));
    }

    // 检查节是否存在
    let value = if config.sections().contains(&section_name.to_string()) {
        if let Some(val) = config.get(section_name, config_item_name) {
            val
        } else {
            println!("WARNING: 配置项名称{}不存在", config_item_name);
            default_value.to_string()
        }
    } else {
        println!("WARNING: 节名称{}不存在", section_name);
        default_value.to_string()
    };

    Ok(if value.is_empty() { default_value.to_string() } else { value })
}

pub async fn write_config_item(
    file: &str,
    section_name: &str,
    config_item_name: &str,
    value: &str,
) -> Result<()> {
    let mut config = Ini::new();
    
    if Path::new(file).exists() {
        // 处理 load 方法可能返回的错误
        if let Err(e) = config.load(file) {
            return Err(anyhow!("加载配置文件失败: {}", e));
        }
    }
    
    // 检查节是否存在
    if !config.sections().contains(&section_name.to_string()) {
        config.set(section_name, "", None);
    }
    
    config.set(section_name, config_item_name, Some(value.to_string()));
    
    // 处理 write 方法可能返回的错误
    if let Err(e) = config.write(file) {
        return Err(anyhow!("写入配置文件失败: {}", e));
    }
    
    Ok(())
}

pub async fn init_config(file: &str, config_map: HashMap<String, HashMap<String, String>>) -> Result<()> {
    let mut ini = Ini::new();
    
    for (section, items) in config_map {
        for (key, value) in items {
            ini.set(&section, &key, Some(value));
        }
    }
    
    // 处理 write 方法可能返回的错误
    if let Err(e) = ini.write(file) {
        return Err(anyhow!("写入配置文件失败: {}", e));
    }
    
    Ok(())
}