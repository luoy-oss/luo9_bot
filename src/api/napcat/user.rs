use reqwest::Client;
use serde_json::json;

// @TODO: 所有请求改为json格式post/get

pub async fn send_private_msg(base_url: &str, access_token: &str, user_id: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("{}/send_private_msg", base_url);
    let client = Client::new();
    
    let params = json!({
        "user_id": user_id,
        "message": message,
        "access_token": access_token
    });
    
    client.post(&url)
        .query(&params)
        .send()
        .await?;
    
    Ok(())
}