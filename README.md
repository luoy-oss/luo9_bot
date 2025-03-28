<div align="center">

# 洛玖机器人 (Rust 版)

_✨ 基于 Rust 重构的洛玖机器人 ✨_

<a href="https://raw.githubusercontent.com/luoy-oss/luo9_bot/main/LICENSE">
    <img src="https://img.shields.io/github/license/luoy-oss/luo9_bot" alt="license">
</a>
<img src="https://img.shields.io/badge/rust-stable-orange?logo=rust" alt="rust">
<br />

</div>

## 运行

> 克隆本仓库
```bash
git clone https://github.com/luoy-oss/luo9_bot.git
```

进入 Rust 版 bot 目录

```bash
cd luo9_bot/rust
```

编译并运行

运行前请进行基础 config 配置

```bash
cargo build --release
.\target\release\luo9_bot.exe
```

## 基础配置
在项目根目录下创建 config.yaml 文件进行配

```yaml
# 机器人基本配置
bot_id: "123456789"  # 机器人QQ号
plugin_path: "./plugins"  # 插件目录
data_path: "./data"  # 数据目录

# API 配置
napcat:
  enable: true  # 启用 NapCat API
base_url: "http://localhost:8080"  # API 基础 URL
access_token: "your_access_token"  # API 访问令牌
```

## 插件编写
在 plugins 目录下新建你的插件

- plugins
  - your_plugin_folder
    - Cargo.toml
    - src/
      - lib.rs
向 plugins/config.yaml 添加插件信息

name 为插件文件夹名称(插件名称)

priority 为插件优先级，建议值：1-65535，值越小，插件优先级越高

enable 为插件是否启用，true 为启用

```yaml
plugins:
  - name: your_plugin_folder
    priority: 10
    enable: true
```

## 插件样例
Cargo.toml 示例:

```toml
[package]
name = "example_plugin"
version = "1.0.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
luo9_bot = { path = "../.." }
anyhow = "1.0"
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
```

lib.rs 必须包含以下内容:

```rust
use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use luo9_bot::config::Value;
use luo9_bot::core::plugin_manager::{Plugin, PluginMetadata};
use luo9_bot::core::message::{GroupMessage, PrivateMessage};
use luo9_bot::export_plugin;
use luo9_bot::api::ApiManager;

// 插件结构体
pub struct YourPlugin {
    metadata: PluginMetadata,
    config: Arc<Value>,
    api: ApiManager,
}

impl YourPlugin {
    // 创建插件实例
    pub fn new(config: Arc<Value>) -> Result<Self> {
        let metadata = PluginMetadata {
            name: "your_plugin".to_string(),
            describe: "你的插件描述".to_string(),
            author: "你的名字".to_string(),
            version: "0.1.0".to_string(),
            message_types: vec![
                "group_message".to_string(),
                "private_message".to_string(),
                // 其他消息类型...
            ],
        };
        
        // 初始化 API
        let api = ApiManager::new()?;
        
        Ok(Self {
            metadata,
            config,
            api,
        })
    }
}

#[async_trait]
impl Plugin for YourPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    // 处理群消息
    async fn handle_group_message(&self, message: &GroupMessage) -> Result<()> {
        // 你的群消息处理逻辑
        if message.content.contains("关键词") {
            match self.api.send_group_message(&message.group_id, "回复内容").await {
                Ok(_) => {},
                Err(e) => return Err(anyhow::anyhow!("发送消息失败: {}", e)),
            }
        }
        
        Ok(())
    }
    
    // 处理私聊消息
    async fn handle_private_message(&self, message: &PrivateMessage) -> Result<()> {
        // 你的私聊消息处理逻辑
        if message.content.contains("关键词") {
            match self.api.send_private_msg(&message.sender_id, "回复内容").await {
                Ok(_) => {},
                Err(e) => return Err(anyhow::anyhow!("发送消息失败: {}", e)),
            }
        }
        
        Ok(())
    }
    
    // 处理戳一戳事件
    async fn handle_group_poke(&self, target_id: &str, user_id: &str, group_id: &str) -> Result<()> {
        // 你的戳一戳处理逻辑
        if target_id == self.config.bot_id.to_string() {
            match self.api.send_group_message(group_id, "别戳我!").await {
                Ok(_) => {},
                Err(e) => return Err(anyhow::anyhow!("发送消息失败: {}", e)),
            }
        }
        
        Ok(())
    }
}

// 创建插件实例的函数
fn create_plugin(config: Arc<Value>) -> Result<Box<dyn Plugin>> {
    let plugin = YourPlugin::new(config)?;
    Ok(Box::new(plugin))
}

// 导出插件创建函数
export_plugin!(create_plugin);
```

## 编译插件
在插件目录下编译插件

```bash
cd plugins/example_plugin
cargo build --release
```
编译后的 DLL 文件将位于 target/release 目录下

## API 使用
在插件中使用 API 发送消息

```rust
// 发送群消息
self.api.send_group_message(&group_id, "消息内容").await {
    Ok(_) => {},
    Err(e) => return Err(anyhow::anyhow!("发送消息失败: {}", e)),
}

// 发送私聊消息
self.api.send_private_msg(&user_id, "消息内容").await {
    Ok(_) => {},
    Err(e) => return Err(anyhow::anyhow!("发送消息失败: {}", e)),
}

// 发送群戳一戳
self.api.send_group_poke(&group_id, &user_id).await {
    Ok(_) => {},
    Err(e) => return Err(anyhow::anyhow!("发送消息失败: {}", e)),
}

// 发送群图片
self.api.send_group_image(&group_id, "图片路径").await {
    Ok(_) => {},
    Err(e) => return Err(anyhow::anyhow!("发送消息失败: {}", e)),
}

// 发送群文件
self.api.send_group_file(&group_id, "文件路径", "文件名", "文件夹ID").await {
    Ok(_) => {},
    Err(e) => return Err(anyhow::anyhow!("发送消息失败: {}", e)),
}
```

# 免责声明
代码仅用于对 Rust 技术的交流学习使用，禁止用于实际生产项目，请勿用于非法用途和商业用途！如因此产生任何法律纠纷，均与作者无关！