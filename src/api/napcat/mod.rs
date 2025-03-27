pub mod group;
pub mod user;

use crate::api::ApiTrait;

pub struct NapCat {
    pub rt: tokio::runtime::Runtime,
    pub base_url: String,
    pub access_token: String,
}

impl NapCat {
    pub fn new(base_url: String, access_token: String) -> Self {
        // 使用tokio::runtime::Builder来创建一个更轻量级的运行时
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        
        Self {
            base_url,
            access_token,
            rt
        }
    }
}

#[async_trait::async_trait]
impl ApiTrait for NapCat {
    async fn send_group_message(&self, group_id: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 在运行时中执行异步函数
        self.rt.block_on(async {
            self::group::send_group_message(&self.base_url, &self.access_token, group_id, message).await 
        })?;
        
        Ok(())
    }

    async fn send_private_msg(&self, user_id: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 在运行时中执行异步函数
        self.rt.block_on(async {
            self::user::send_private_msg(&self.base_url, &self.access_token, user_id, message).await
        })?;
        
        Ok(())
    }

    async fn send_group_ai_record(&self, group_id: &str, voice: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 在运行时中执行异步函数
        self.rt.block_on(async {
            self::group::send_group_ai_record(&self.base_url, &self.access_token, group_id, voice, message).await
        })?;
        
        Ok(())
    }

    async fn send_group_at(&self, group_id: &str, qq: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 在运行时中执行异步函数
        self.rt.block_on(async {
            self::group::send_group_at(&self.base_url, &self.access_token, group_id, qq).await
        })?;
        
        Ok(())
    }

    async fn send_group_image(&self, group_id: &str, file: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 在运行时中执行异步函数
        self.rt.block_on(async {
            self::group::send_group_image(&self.base_url, &self.access_token, group_id, file).await
        })?;
        
        Ok(())


    }

    async fn send_group_file(&self, group_id: &str, file: &str, name: &str, folder_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 在运行时中执行异步函数
        self.rt.block_on(async {
            self::group::upload_group_file(&self.base_url, &self.access_token, group_id, file, name, folder_id).await
        })?;
        
        Ok(())
    }

    async fn send_group_poke(&self, group_id: &str, user_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 在运行时中执行异步函数
        self.rt.block_on(async {
            self::group::send_group_poke(&self.base_url, &self.access_token, group_id, user_id).await
        })?;
        
        Ok(())
    }

    async fn get_group_files_by_folder(&self, group_id: &str, folder_id: &str, file_count: i32) -> Result<String, Box<dyn std::error::Error>> {
        // 在运行时中执行异步函数
        let res = self.rt.block_on(async {
            self::group::get_group_files_by_folder(&self.base_url, &self.access_token, group_id, folder_id, file_count).await
        })?;
        
        Ok(res)
    }

    async fn get_group_root_files(&self, group_id: &str) -> Result<String, Box<dyn std::error::Error>> {
        // 在运行时中执行异步函数
        let res = self.rt.block_on(async {
            self::group::get_group_root_files(&self.base_url, &self.access_token, group_id).await
        })?;

        Ok(res)
    }
}