use anyhow::Result;
use md5::{Digest, Md5};
use reqwest;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use tokio::runtime::Runtime;

pub async fn calculate_file_hash(file_path: &str) -> Result<String> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let mut hasher = Md5::new();
    hasher.update(&buffer);
    let result = hasher.finalize();
    
    Ok(format!("{:x}", result))
}

pub async fn download_image_if_needed(_message: &str, img_url: &str, save_path: &str) -> Result<()> {
    // 创建一个 Tokio 运行时
    let rt = Runtime::new()?;
    
    // 在 Tokio 运行时中执行异步操作
    rt.block_on(async {
        // 确保父目录存在
        if let Some(parent) = Path::new(save_path).parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        if Path::new(save_path).exists() {
            println!("图片已存在，检查是否需要更新");
            // 计算本地图片的哈希值
            let local_hash = calculate_file_hash(save_path).await?;
            
            // 下载网络图片到临时文件
            let temp_path = format!("{}.temp", save_path);
            
            // 使用异步的 reqwest 客户端
            let client = reqwest::Client::new();
            let response = client.get(img_url).send().await?;
            let bytes = response.bytes().await?;
            
            let mut file = File::create(&temp_path)?;
            file.write_all(&bytes)?;
            
            // 计算网络图片的哈希值
            let remote_hash = calculate_file_hash(&temp_path).await?;
            
            if local_hash == remote_hash {
                // 如果哈希值一致，删除临时文件，无需下载
                fs::remove_file(&temp_path)?;
                println!("图片已存在且一致，无需更新");
                return Ok(());
            } else {
                // 如果哈希值不一致，替换本地图片
                fs::rename(&temp_path, save_path)?;
                println!("图片已更新");
            }
        } else {
            println!("图片不存在，开始下载");
            // 如果本地图片不存在，直接下载
            
            // 使用异步的 reqwest 客户端
            let client = reqwest::Client::new();
            let response = client.get(img_url).send().await?;
            let bytes = response.bytes().await?;
            
            let mut file = File::create(save_path)?;
            file.write_all(&bytes)?;
            println!("图片已下载");
        }
        
        Ok(())
    })
}