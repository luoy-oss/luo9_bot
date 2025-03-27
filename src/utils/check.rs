use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use crate::utils::ini_files;

pub async fn at_check(message: &str, bot_id: &str) -> bool {
    message.contains(&format!("[CQ:at,qq={}]", bot_id))
}

pub async fn duplicate_message_check(message: &str, group_id: &str, count: usize) -> bool {
    // 这里需要实现消息重复检测的逻辑
    // 由于原Python代码没有完整提供，这里只是一个框架
    // 实际实现需要存储和检查最近的消息
    true
}

pub async fn register_check(group_id: &str, qq: &str, user_data_path: &str) -> bool {
    if Path::new(user_data_path).exists() {
        let result = ini_files::read_config_item(
            user_data_path,
            "注册",
            "是否注册",
            "否"
        ).await.unwrap_or_else(|_| "否".to_string());
        
        return result == "是";
    }
    false
}

pub async fn frozen_check(group_id: &str, qq: &str, admin_frozen_path: &str) -> bool {
    if Path::new(admin_frozen_path).exists() {
        let result = ini_files::read_config_item(
            admin_frozen_path,
            "冻结账号",
            qq,
            "否"
        ).await.unwrap_or_else(|_| "否".to_string());
        
        return result == "是";
    }
    false
}

pub async fn interactiveState_check() -> bool {
    // 这里需要实现交互状态检查的逻辑
    // 由于原Python代码没有完整提供，这里默认返回true
    true
}

pub async fn data_path_check(group_id: &str, user_id: &str) -> HashMap<String, String> {
    // 这里需要实现数据路径检查的逻辑
    // 返回包含各种路径的HashMap
    let mut paths = HashMap::new();
    paths.insert("USER_DATA_PATH".to_string(), format!("data/users/{}/{}.ini", group_id, user_id));
    paths.insert("ADMIN_PRIORITY_PATH".to_string(), format!("data/admin/{}/priority.ini", group_id));
    paths.insert("ADMIN_FROZEN_PATH".to_string(), format!("data/admin/{}/frozen.ini", group_id));
    
    paths
}