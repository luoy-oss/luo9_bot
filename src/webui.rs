use axum::{
    Router,
    extract::{Multipart, Path, Query, Request, State},
    http::{StatusCode, header},
    middleware::{self, Next},
    response::{
        Html, IntoResponse, Json, Response,
        sse::{Event, Sse},
    },
    routing::{delete, get, post, put},
};
use futures_util::stream::Stream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

use crate::utils::logger;

const REGISTRY_URL: &str = "https://raw.githubusercontent.com/luo9-bot/registry/main/registry.json";

/// GitHub 资源镜像前缀列表（按优先级排序）
/// 每个前缀会拼接在原始 URL 的前面（去掉 https:// 前缀）
const GITHUB_RAW_MIRRORS: &[&str] = &[
    "https://ghfast.top/",
    "https://ghproxy.cn/",
    "https://raw.gitmirror.com/",
];

/// GitHub release 下载镜像前缀列表
const GITHUB_RELEASE_MIRRORS: &[&str] = &[
    "https://ghfast.top/",
    "https://ghproxy.cn/",
    "https://mirror.ghproxy.com/",
];

/// HTTP 请求超时时间
const HTTP_TIMEOUT_SECS: u64 = 15;

// ========== 注册表数据结构 ==========

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Registry {
    plugins: HashMap<String, RegistryPlugin>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct RegistryPlugin {
    description: String,
    repo: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    versions: Vec<RegistryVersion>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct RegistryVersion {
    version: String,
    tag: String,
    #[serde(default)]
    sdk_version: String,
    assets: HashMap<String, String>,
}

// ========== WebUI 数据结构 ==========

/// 下载进度事件
#[derive(Debug, Clone, Serialize)]
pub struct DownloadProgress {
    pub plugin_name: String,
    pub status: String,      // "downloading", "success", "error"
    pub message: String,
    pub progress: Option<f32>, // 0.0 - 1.0，None 表示不确定
}

pub struct WebuiState {
    pub plugin_dir: String,
    pub start_time: std::time::Instant,
    pub token: String,
    pub progress_tx: broadcast::Sender<DownloadProgress>,
}

#[derive(Serialize, Deserialize)]
struct PluginInfo {
    name: String,
    file: String,
    enabled: bool,
    #[serde(default)]
    priority: i32,
    #[serde(default)]
    block_enabled: bool,
    #[serde(default)]
    active: bool,
}

#[derive(Serialize)]
struct StatusResponse {
    uptime_secs: u64,
    plugin_count: usize,
    plugins: Vec<PluginInfo>,
    plugin_dir: String,
}

#[derive(Deserialize)]
struct LogQuery {
    after: Option<usize>,
}

#[derive(Serialize)]
struct LogResponse {
    lines: Vec<String>,
    total: usize,
}

#[derive(Serialize)]
struct MsgResponse {
    ok: bool,
    message: String,
}

#[derive(Deserialize)]
struct PriorityRequest {
    priority: i32,
}

#[derive(Deserialize)]
struct BlockRequest {
    block_enabled: bool,
}

/// 注册表中可用插件的展示信息
#[derive(Serialize)]
struct AvailablePlugin {
    name: String,
    description: String,
    repo: String,
    #[serde(default)]
    tags: Vec<String>,
    latest_version: String,
    #[serde(default)]
    sdk_version: String,
    installed: bool,
    installed_version: Option<String>,
    #[serde(default)]
    versions: Vec<RegistryVersion>,
}

/// 配置更新请求
#[derive(Deserialize)]
struct ConfigUpdateRequest {
    napcat: Option<NapcatConfigReq>,
    logging: Option<LoggingConfigReq>,
    plugins: Option<PluginConfigReq>,
    webui: Option<WebuiConfigReq>,
}

#[derive(Deserialize)]
struct NapcatConfigReq {
    ws_client_host: Option<String>,
    ws_client_port: Option<u16>,
    ws_server_host: Option<String>,
    ws_server_port: Option<u16>,
    timeout_seconds: Option<u64>,
    token: Option<String>,
}

#[derive(Deserialize)]
struct LoggingConfigReq {
    level: Option<String>,
}

#[derive(Deserialize)]
struct PluginConfigReq {
    enabled: Option<bool>,
    plugin_dir: Option<String>,
    auto_load: Option<bool>,
}

#[derive(Deserialize)]
struct WebuiConfigReq {
    host: Option<String>,
    port: Option<u16>,
    token: Option<String>,
}

#[derive(Deserialize)]
struct RawConfigUpdate {
    content: String,
}

// ========== 启动服务 ==========

/// 生成随机 token（16 位十六进制）
fn generate_token() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    // 简单的伪随机：时间戳的低位 + 进程 ID
    let pid = std::process::id();
    format!("{:016x}", timestamp.wrapping_mul(pid as u128))
}

pub async fn start(host: &str, port: u16, plugin_dir: String, config_token: String) {
    // token 生成逻辑：配置为空时随机生成
    let token = if config_token.is_empty() {
        let generated = generate_token();
        info!("WebUI token 未配置，已自动生成: {}", generated);
        generated
    } else {
        info!("WebUI 使用配置的 token");
        config_token
    };

    // 创建下载进度广播通道
    let (progress_tx, _) = broadcast::channel::<DownloadProgress>(100);

    let state = Arc::new(WebuiState {
        plugin_dir,
        start_time: std::time::Instant::now(),
        token: token.clone(),
        progress_tx,
    });

    // 静态资源无需鉴权，API 需要鉴权
    let static_routes = Router::new()
        .route("/", get(index_page))
        .route("/style.css", get(style_css))
        .route("/app.js", get(app_js));

    let api_routes = Router::new()
        .route("/api/status", get(api_status))
        .route("/api/plugins", get(api_plugins))
        .route("/api/plugins/upload", post(api_plugin_upload))
        .route("/api/plugins/{name}", delete(api_plugin_delete))
        .route("/api/plugins/{name}/enable", post(api_plugin_enable))
        .route("/api/plugins/{name}/disable", post(api_plugin_disable))
        .route("/api/plugins/{name}/reload", post(api_plugin_reload))
        .route("/api/plugins/{name}/priority", put(api_plugin_priority))
        .route("/api/plugins/{name}/block", put(api_plugin_block))
        .route("/api/plugins/install/{name}", post(api_plugin_install))
        .route("/api/registry", get(api_registry))
        .route("/api/logs", get(api_logs))
        .route("/api/config/path", get(api_config_path))
        .route("/api/config", get(api_config_get).put(api_config_put))
        .route("/api/config/raw", get(api_config_raw_get).put(api_config_raw_put))
        .route("/api/download-progress", get(api_download_progress))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    let app = Router::new()
        .merge(static_routes)
        .merge(api_routes)
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("{}:{}", host, port);
    info!("WebUI 启动于 http://{}?token={}", addr, token);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// ========== 鉴权中间件 ==========

/// 从 query 字符串中提取指定参数值
fn extract_query_param(query: &str, key: &str) -> Option<String> {
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            if k == key {
                return Some(v.to_string());
            }
        }
    }
    None
}

/// API 鉴权中间件：检查 query 参数或 cookie 中的 token
async fn auth_middleware(
    State(state): State<Arc<WebuiState>>,
    request: Request,
    next: Next,
) -> Response {
    let uri = request.uri().clone();
    let headers = request.headers().clone();

    // 1. 从 query 参数获取 token
    let query_token = uri
        .query()
        .and_then(|q| extract_query_param(q, "token"));

    // 2. 从 cookie 获取 token
    let cookie_token = headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|c| {
                let c = c.trim();
                if c.starts_with("luo9_token=") {
                    Some(c.trim_start_matches("luo9_token=").to_string())
                } else {
                    None
                }
            })
        });

    // 验证 token
    let valid = query_token
        .as_deref()
        .or(cookie_token.as_deref())
        .map(|t| t == state.token)
        .unwrap_or(false);

    if !valid {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"ok": false, "message": "未授权访问，请提供有效的 token"})),
        )
            .into_response();
    }

    let mut response = next.run(request).await;

    // 如果是 query 参数验证通过，设置 cookie
    if query_token.is_some() {
        let cookie_value = format!("luo9_token={}; Path=/; HttpOnly; SameSite=Strict", state.token);
        response.headers_mut().insert(
            header::SET_COOKIE,
            cookie_value.parse().unwrap(),
        );
    }

    response
}

// ========== 静态资源 ==========

async fn index_page() -> impl IntoResponse {
    Html(include_str!("webui/index.html"))
}

async fn style_css() -> impl IntoResponse {
    (
        [(axum::http::header::CONTENT_TYPE, "text/css; charset=utf-8")],
        include_str!("webui/style.css"),
    )
}

async fn app_js() -> impl IntoResponse {
    (
        [(axum::http::header::CONTENT_TYPE, "application/javascript; charset=utf-8")],
        include_str!("webui/app.js"),
    )
}

// ========== 状态 API ==========

async fn api_status(State(state): State<Arc<WebuiState>>) -> impl IntoResponse {
    let plugins = scan_plugins(&state.plugin_dir);
    let resp = StatusResponse {
        uptime_secs: state.start_time.elapsed().as_secs(),
        plugin_count: plugins.len(),
        plugins,
        plugin_dir: state.plugin_dir.clone(),
    };
    Json(resp)
}

async fn api_plugins(State(state): State<Arc<WebuiState>>) -> impl IntoResponse {
    let mut plugins = scan_plugins(&state.plugin_dir);

    // 从插件管理器获取运行时信息
    let manager = crate::plugin::GLOBAL_PLUGIN_MANAGER.lock().await;
    for plugin in &mut plugins {
        if let Some(info) = manager.get_plugin_info(&plugin.name) {
            plugin.priority = info.priority;
            plugin.block_enabled = info.block_enabled;
            plugin.active = info.active;
        }
    }

    Json(plugins)
}

/// 返回当前使用的配置文件路径
async fn api_config_path() -> impl IntoResponse {
    let path = crate::config::LNConfig::config_path();
    Json(serde_json::json!({
        "path": path.to_string_lossy(),
    }))
}

// ========== 注册表 API ==========

async fn api_registry(State(state): State<Arc<WebuiState>>) -> impl IntoResponse {
    let registry = match fetch_registry().await {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({"ok": false, "message": format!("获取注册表失败: {e}")})),
            )
                .into_response();
        }
    };

    let installed = scan_plugins(&state.plugin_dir);
    let installed_map: HashMap<String, &PluginInfo> =
        installed.iter().map(|p| (p.name.clone(), p)).collect();

    let mut available: Vec<AvailablePlugin> = Vec::new();
    for (name, plugin) in &registry.plugins {
        let latest = plugin.versions.first();
        let inst = installed_map.get(name.as_str());
        available.push(AvailablePlugin {
            name: name.clone(),
            description: plugin.description.clone(),
            repo: plugin.repo.clone(),
            tags: plugin.tags.clone(),
            latest_version: latest.map(|v| v.version.clone()).unwrap_or_default(),
            sdk_version: latest.map(|v| v.sdk_version.clone()).unwrap_or_default(),
            installed: inst.is_some(),
            installed_version: inst.map(|_| {
                installed_map
                    .get(name.as_str())
                    .map(|p| p.file.clone())
                    .unwrap_or_default()
            }),
            versions: plugin.versions.clone(),
        });
    }

    // 按名称排序
    available.sort_by(|a, b| a.name.cmp(&b.name));
    Json(available).into_response()
}

#[derive(Deserialize)]
struct InstallQuery {
    version: Option<String>,
}

/// 从注册表安装插件
async fn api_plugin_install(
    State(state): State<Arc<WebuiState>>,
    Path(name): Path<String>,
    Query(q): Query<InstallQuery>,
) -> impl IntoResponse {
    let registry = match fetch_registry().await {
        Ok(r) => r,
        Err(e) => return json_err(StatusCode::BAD_GATEWAY, &format!("获取注册表失败: {e}")),
    };

    let plugin = match registry.plugins.get(&name) {
        Some(p) => p,
        None => return json_err(StatusCode::NOT_FOUND, &format!("插件 {name} 不在注册表中")),
    };

    // 查找指定版本，或使用最新版本
    let version_entry = if let Some(ref ver) = q.version {
        match plugin.versions.iter().find(|v| v.version == *ver) {
            Some(v) => v,
            None => return json_err(StatusCode::NOT_FOUND, &format!("插件 {name} 没有版本 {ver}")),
        }
    } else {
        match plugin.versions.first() {
            Some(v) => v,
            None => return json_err(StatusCode::NOT_FOUND, &format!("插件 {name} 没有可用版本")),
        }
    };

    // 确定平台和资源文件名
    let platform_key = detect_platform();
    let asset_name = match version_entry.assets.get(&platform_key) {
        Some(a) => a.clone(),
        None => {
            return json_err(
                StatusCode::BAD_REQUEST,
                &format!("插件 {name} v{} 不支持当前平台 {platform_key}", version_entry.version),
            )
        }
    };

    // 构建下载 URL（含镜像 fallback）
    let primary_url = format!(
        "https://github.com/{}/releases/download/{}/{}",
        plugin.repo, version_entry.tag, asset_name
    );
    let download_urls = build_mirrored_urls(&primary_url, GITHUB_RELEASE_MIRRORS);

    info!("正在下载插件: {} v{}", name, version_entry.version);

    // 发送下载开始进度
    let _ = state.progress_tx.send(DownloadProgress {
        plugin_name: name.clone(),
        status: "downloading".to_string(),
        message: format!("正在下载 {} v{}...", name, version_entry.version),
        progress: Some(0.0),
    });

    // 带镜像 fallback 下载文件
    let bytes = match download_with_fallback(&download_urls, &state.progress_tx, &name).await {
        Ok(b) => b,
        Err(e) => {
            let _ = state.progress_tx.send(DownloadProgress {
                plugin_name: name.clone(),
                status: "error".to_string(),
                message: format!("下载失败: {e}"),
                progress: None,
            });
            return json_err(StatusCode::BAD_GATEWAY, &format!("下载失败: {e}"));
        }
    };

    // 发送下载完成进度
    let _ = state.progress_tx.send(DownloadProgress {
        plugin_name: name.clone(),
        status: "downloading".to_string(),
        message: format!("正在保存文件..."),
        progress: Some(0.9),
    });

    // 保存到插件目录
    let dir = PathBuf::from(&state.plugin_dir);
    if !dir.exists() {
        if let Err(e) = fs::create_dir_all(&dir) {
            return json_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("创建目录失败: {e}"));
        }
    }

    let target = dir.join(&asset_name);
    if let Err(e) = fs::write(&target, &bytes) {
        return json_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("保存文件失败: {e}"));
    }

    info!("插件安装成功: {} v{} -> {}", name, version_entry.version, target.display());

    // 自动加载插件（无需重启）
    let config = crate::config::LNConfig::load();
    let config_entries = config.as_ref().map(|c| c.plugins.plugins.clone()).unwrap_or_default();
    match crate::plugin::enable_plugin(&name, &target, &config_entries).await {
        Ok(msg) => {
            info!("插件 {} 已自动加载: {}", name, msg);
            let _ = state.progress_tx.send(DownloadProgress {
                plugin_name: name.clone(),
                status: "success".to_string(),
                message: format!("插件 {} v{} 安装成功并已加载", name, version_entry.version),
                progress: Some(1.0),
            });
            json_ok(&format!(
                "插件 {} v{} 安装成功并已加载",
                name, version_entry.version
            ))
        }
        Err(e) => {
            warn!("插件 {} 安装成功但自动加载失败: {}", name, e);
            let _ = state.progress_tx.send(DownloadProgress {
                plugin_name: name.clone(),
                status: "success".to_string(),
                message: format!("插件 {} v{} 安装成功，但自动加载失败: {}", name, version_entry.version, e),
                progress: Some(1.0),
            });
            json_ok(&format!(
                "插件 {} v{} 安装成功，但自动加载失败: {}",
                name, version_entry.version, e
            ))
        }
    }
}

// ========== 插件管理 API ==========

async fn api_plugin_enable(
    State(state): State<Arc<WebuiState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let dir = PathBuf::from(&state.plugin_dir);

    let disabled_path = find_disabled_file(&dir, &name);
    let Some(disabled_path) = disabled_path else {
        return json_err(StatusCode::NOT_FOUND, &format!("未找到禁用的插件 {name}"));
    };

    let original_name = disabled_path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .trim_end_matches(".disabled")
        .to_string();
    let enabled_path = dir.join(&original_name);

    // 重命名文件
    if let Err(e) = fs::rename(&disabled_path, &enabled_path) {
        return json_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("启用失败: {e}"));
    }

    // 运行时加载插件
    let config = crate::config::LNConfig::load();
    let config_entries = config.as_ref().map(|c| c.plugins.plugins.clone()).unwrap_or_default();
    match crate::plugin::enable_plugin(&name, &enabled_path, &config_entries).await {
        Ok(msg) => {
            info!("{}", msg);
            json_ok(&format!("插件 {name} 已启用并加载"))
        }
        Err(e) => {
            warn!("插件 {name} 文件已恢复但运行时加载失败: {e}");
            json_ok(&format!("插件 {name} 文件已恢复，但运行时加载失败: {e}"))
        }
    }
}

async fn api_plugin_disable(
    State(state): State<Arc<WebuiState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let dir = PathBuf::from(&state.plugin_dir);

    let enabled_path = find_enabled_file(&dir, &name);
    let Some(enabled_path) = enabled_path else {
        return json_err(StatusCode::NOT_FOUND, &format!("未找到插件 {name}"));
    };

    let file_name = enabled_path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let disabled_path = dir.join(format!("{file_name}.disabled"));

    // 运行时禁用（取消订阅，等待线程退出，释放 DLL 锁）
    {
        let mut manager = crate::plugin::GLOBAL_PLUGIN_MANAGER.lock().await;
        match manager.disable_plugin(&name).await {
            Ok(msg) => info!("{}", msg),
            Err(e) => {
                // 如果不是"已经禁用"的错误，则中止文件重命名
                if !e.contains("已经是禁用状态") {
                    return json_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("运行时禁用失败: {e}"));
                }
                warn!("插件 {} 运行时状态: {}", name, e);
            }
        }
    }

    // 文件重命名（持久化）
    if let Err(e) = fs::rename(&enabled_path, &disabled_path) {
        return json_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("禁用失败: {e}"));
    }

    info!("插件已禁用: {name}");
    json_ok(&format!("插件 {name} 已禁用"))
}

/// 热重载插件
async fn api_plugin_reload(
    Path(name): Path<String>,
) -> impl IntoResponse {
    let config = crate::config::LNConfig::load();
    let config_entries = config.as_ref().map(|c| c.plugins.plugins.clone()).unwrap_or_default();

    match crate::plugin::reload_plugin(&name, &config_entries).await {
        Ok(msg) => {
            info!("{}", msg);
            json_ok(&msg)
        }
        Err(e) => {
            json_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("热重载失败: {e}"))
        }
    }
}

/// 设置插件优先级
async fn api_plugin_priority(
    Path(name): Path<String>,
    Json(req): Json<PriorityRequest>,
) -> impl IntoResponse {
    let mut manager = crate::plugin::GLOBAL_PLUGIN_MANAGER.lock().await;
    match manager.update_priority(&name, req.priority) {
        Ok(()) => {
            // 更新分发列表
            let entries = manager.get_dispatch_list();
            drop(manager);
            crate::plugin::update_dispatch_list(entries);

            // 持久化到配置文件
            let mut config = match crate::config::LNConfig::load() {
                Ok(c) => c,
                Err(e) => return json_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("加载配置失败: {e}")),
            };
            config.upsert_plugin_entry(crate::config::PluginEntry {
                name: name.clone(),
                priority: req.priority,
                block_enabled: config.get_plugin_entry(&name).map(|e| e.block_enabled).unwrap_or(false),
            });
            if let Err(e) = config.save() {
                return json_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("保存配置失败: {e}"));
            }

            json_ok(&format!("插件 {name} 优先级已设置为 {}", req.priority))
        }
        Err(e) => json_err(StatusCode::NOT_FOUND, &e),
    }
}

/// 设置插件消息阻断
async fn api_plugin_block(
    Path(name): Path<String>,
    Json(req): Json<BlockRequest>,
) -> impl IntoResponse {
    let mut manager = crate::plugin::GLOBAL_PLUGIN_MANAGER.lock().await;
    match manager.update_block(&name, req.block_enabled) {
        Ok(()) => {
            // 更新分发列表
            let entries = manager.get_dispatch_list();
            drop(manager);
            crate::plugin::update_dispatch_list(entries);

            // 持久化到配置文件
            let mut config = match crate::config::LNConfig::load() {
                Ok(c) => c,
                Err(e) => return json_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("加载配置失败: {e}")),
            };
            config.upsert_plugin_entry(crate::config::PluginEntry {
                name: name.clone(),
                priority: config.get_plugin_entry(&name).map(|e| e.priority).unwrap_or(0),
                block_enabled: req.block_enabled,
            });
            if let Err(e) = config.save() {
                return json_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("保存配置失败: {e}"));
            }

            let status = if req.block_enabled { "启用" } else { "禁用" };
            json_ok(&format!("插件 {name} 消息阻断已{status}"))
        }
        Err(e) => json_err(StatusCode::NOT_FOUND, &e),
    }
}

async fn api_plugin_upload(
    State(state): State<Arc<WebuiState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let dir = PathBuf::from(&state.plugin_dir);
    if !dir.exists() {
        if let Err(e) = fs::create_dir_all(&dir) {
            return json_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("创建目录失败: {e}"));
        }
    }

    let mut saved_count = 0;
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let file_name = match field.file_name() {
            Some(name) => name.to_string(),
            None => continue,
        };

        let lower = file_name.to_lowercase();
        if !lower.ends_with(".dll") && !lower.ends_with(".so") {
            continue;
        }

        let data = match field.bytes().await {
            Ok(d) => d,
            Err(_) => continue,
        };

        let target = dir.join(&file_name);
        if let Err(e) = fs::write(&target, &data) {
            return json_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("保存文件失败: {e}"));
        }

        info!("插件已上传: {} ({} 字节)", file_name, data.len());
        saved_count += 1;
    }

    if saved_count == 0 {
        return json_err(
            StatusCode::BAD_REQUEST,
            "未收到有效的插件文件（需要 .dll 或 .so）",
        );
    }

    json_ok(&format!(
        "已上传 {saved_count} 个插件文件，重启后生效"
    ))
}

async fn api_plugin_delete(
    State(state): State<Arc<WebuiState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let dir = PathBuf::from(&state.plugin_dir);

    let enabled = find_enabled_file(&dir, &name);
    let disabled = find_disabled_file(&dir, &name);

    if enabled.is_none() && disabled.is_none() {
        return json_err(StatusCode::NOT_FOUND, &format!("未找到插件 {name}"));
    }

    let mut deleted = 0;
    if let Some(path) = enabled {
        if fs::remove_file(&path).is_ok() {
            info!("已删除插件文件: {}", path.display());
            deleted += 1;
        }
    }
    if let Some(path) = disabled {
        if fs::remove_file(&path).is_ok() {
            info!("已删除禁用插件文件: {}", path.display());
            deleted += 1;
        }
    }

    json_ok(&format!(
        "已删除插件 {name} 的 {deleted} 个文件，重启后生效"
    ))
}

// ========== 日志 API ==========

async fn api_logs(Query(q): Query<LogQuery>) -> impl IntoResponse {
    let all = logger::get_logs();
    let total = all.len();
    let after = q.after.unwrap_or(0);

    let lines = if after < total {
        all[after..].to_vec()
    } else {
        Vec::new()
    };

    Json(LogResponse { lines, total })
}

// ========== 配置 API ==========

/// 获取当前配置（JSON 格式）
async fn api_config_get() -> impl IntoResponse {
    match crate::config::LNConfig::load() {
        Ok(config) => Json(serde_json::json!({
            "ok": true,
            "config": {
                "napcat": {
                    "ws_client_host": config.napcat.ws_client_host,
                    "ws_client_port": config.napcat.ws_client_port,
                    "ws_server_host": config.napcat.ws_server_host,
                    "ws_server_port": config.napcat.ws_server_port,
                    "timeout_seconds": config.napcat.timeout_seconds,
                    "token": config.napcat.token,
                },
                "logging": {
                    "level": config.logging.level,
                },
                "plugins": {
                    "enabled": config.plugins.enabled,
                    "plugin_dir": config.plugins.plugin_dir,
                    "auto_load": config.plugins.auto_load,
                },
                "webui": {
                    "host": config.webui.host,
                    "port": config.webui.port,
                    "token": config.webui.token,
                },
            }
        })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"ok": false, "message": format!("加载配置失败: {e}")})),
        ).into_response(),
    }
}

/// 更新配置（JSON 格式，合并更新）
async fn api_config_put(Json(req): Json<ConfigUpdateRequest>) -> impl IntoResponse {
    let mut config = match crate::config::LNConfig::load() {
        Ok(c) => c,
        Err(e) => return json_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("加载配置失败: {e}")),
    };

    if let Some(napcat) = req.napcat {
        if let Some(v) = napcat.ws_client_host { config.napcat.ws_client_host = v; }
        if let Some(v) = napcat.ws_client_port { config.napcat.ws_client_port = v; }
        if let Some(v) = napcat.ws_server_host { config.napcat.ws_server_host = v; }
        if let Some(v) = napcat.ws_server_port { config.napcat.ws_server_port = v; }
        if let Some(v) = napcat.timeout_seconds { config.napcat.timeout_seconds = v; }
        if let Some(v) = napcat.token { config.napcat.token = v; }
    }

    if let Some(logging) = req.logging {
        if let Some(v) = logging.level { config.logging.level = v; }
    }

    if let Some(plugins) = req.plugins {
        if let Some(v) = plugins.enabled { config.plugins.enabled = v; }
        if let Some(v) = plugins.plugin_dir { config.plugins.plugin_dir = v; }
        if let Some(v) = plugins.auto_load { config.plugins.auto_load = v; }
    }

    if let Some(webui) = req.webui {
        if let Some(v) = webui.host { config.webui.host = v; }
        if let Some(v) = webui.port { config.webui.port = v; }
        if let Some(v) = webui.token { config.webui.token = v; }
    }

    if let Err(e) = config.save() {
        return json_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("保存配置失败: {e}"));
    }

    info!("配置已更新");
    json_ok("配置已保存")
}

/// 获取原始 TOML 配置
async fn api_config_raw_get() -> impl IntoResponse {
    let path = crate::config::LNConfig::config_path();
    match fs::read_to_string(&path) {
        Ok(content) => Json(serde_json::json!({
            "ok": true,
            "path": path.to_string_lossy(),
            "content": content,
        })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"ok": false, "message": format!("读取配置失败: {e}")})),
        ).into_response(),
    }
}

/// 更新原始 TOML 配置
async fn api_config_raw_put(Json(req): Json<RawConfigUpdate>) -> impl IntoResponse {
    // 验证 TOML 格式是否合法
    if let Err(e) = toml::from_str::<crate::config::LNConfig>(&req.content) {
        return json_err(StatusCode::BAD_REQUEST, &format!("TOML 格式错误: {e}"));
    }

    let path = crate::config::LNConfig::config_path();
    if let Err(e) = fs::write(&path, &req.content) {
        return json_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("写入配置失败: {e}"));
    }

    info!("原始配置已更新");
    json_ok("配置已保存，重启后生效")
}

// ========== 下载进度 SSE ==========

/// 下载进度 SSE 端点
async fn api_download_progress(
    State(state): State<Arc<WebuiState>>,
) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
    let mut rx = state.progress_tx.subscribe();

    let stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(progress) => {
                    let data = serde_json::to_string(&progress).unwrap_or_default();
                    yield Ok(Event::default().data(data));
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("ping"),
    )
}

// ========== 辅助函数 ==========

fn json_ok(msg: &str) -> (StatusCode, Json<MsgResponse>) {
    (
        StatusCode::OK,
        Json(MsgResponse {
            ok: true,
            message: msg.to_string(),
        }),
    )
}

fn json_err(status: StatusCode, msg: &str) -> (StatusCode, Json<MsgResponse>) {
    (
        status,
        Json(MsgResponse {
            ok: false,
            message: msg.to_string(),
        }),
    )
}

/// 构建镜像 URL 列表：各镜像 URL + 原始 URL（镜像优先）
///
/// 对于 `https://github.com/...` 形式的 URL，
/// 镜像 URL 为 `https://ghfast.top/https://github.com/...`
fn build_mirrored_urls(original: &str, mirrors: &[&str]) -> Vec<String> {
    let mut urls = Vec::with_capacity(1 + mirrors.len());
    // 镜像优先，避免直连 GitHub 超时
    for mirror in mirrors {
        urls.push(format!("{}{}", mirror, original));
    }
    urls.push(original.to_string());
    urls
}

/// 带镜像 fallback 的 HTTP GET 请求
///
/// 依次尝试每个 URL，返回第一个成功的响应文本。
/// 所有 URL 都失败时，返回最后一个错误。
async fn fetch_with_fallback(urls: &[String]) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(HTTP_TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

    let mut last_err = String::new();
    for url in urls {
        match client
            .get(url)
            .header("User-Agent", "luo9-bot")
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                match resp.text().await {
                    Ok(body) => return Ok(body),
                    Err(e) => last_err = format!("读取响应失败 ({url}): {e}"),
                }
            }
            Ok(resp) => {
                last_err = format!("HTTP {} ({url})", resp.status());
                warn!("镜像请求失败: {}", last_err);
            }
            Err(e) => {
                last_err = format!("请求失败 ({url}): {e}");
                warn!("镜像请求失败: {}", last_err);
            }
        }
    }
    Err(last_err)
}

/// 带镜像 fallback 的文件下载
///
/// 依次尝试每个 URL，返回第一个成功的响应字节。
/// 支持进度推送。
async fn download_with_fallback(
    urls: &[String],
    progress_tx: &broadcast::Sender<DownloadProgress>,
    plugin_name: &str,
) -> Result<Vec<u8>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(HTTP_TIMEOUT_SECS * 2)) // 缩短超时，快速 fallback
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

    let total_urls = urls.len();
    let mut last_err = String::new();

    for (i, url) in urls.iter().enumerate() {
        // 发送尝试进度
        let progress = (i as f32) / (total_urls as f32) * 0.8; // 0% - 80%
        let is_mirror = url.contains("ghfast.top") || url.contains("ghproxy.cn") || url.contains("gitmirror.com") || url.contains("mirror.ghproxy.com");
        let source = if is_mirror { "镜像" } else { "直连" };
        let _ = progress_tx.send(DownloadProgress {
            plugin_name: plugin_name.to_string(),
            status: "downloading".to_string(),
            message: format!("正在尝试{source}: {}", if is_mirror { url.split('/').nth(2).unwrap_or("...") } else { "github.com" }),
            progress: Some(progress),
        });

        match client
            .get(url)
            .header("User-Agent", "luo9-bot")
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                // 获取 Content-Length 用于进度显示
                let total_size = resp.content_length().unwrap_or(0);

                // 流式下载，支持进度更新
                match resp.bytes().await {
                    Ok(chunk) => {
                        let bytes = chunk.to_vec();
                        let downloaded = bytes.len() as u64;

                        // 发送下载进度
                        let progress = if total_size > 0 {
                            0.8 + (downloaded as f32 / total_size as f32) * 0.1 // 80% - 90%
                        } else {
                            0.85
                        };
                        let _ = progress_tx.send(DownloadProgress {
                            plugin_name: plugin_name.to_string(),
                            status: "downloading".to_string(),
                            message: format!("已下载 {:.1} KB", downloaded as f64 / 1024.0),
                            progress: Some(progress),
                        });

                        return Ok(bytes);
                    }
                    Err(e) => {
                        last_err = format!("读取响应失败 ({url}): {e}");
                        warn!("镜像下载失败: {}", last_err);
                    }
                }
            }
            Ok(resp) => {
                last_err = format!("HTTP {} ({url})", resp.status());
                warn!("镜像下载失败: {}", last_err);
            }
            Err(e) => {
                last_err = format!("请求失败 ({url}): {e}");
                warn!("镜像下载失败: {}", last_err);
            }
        }
    }
    Err(last_err)
}

async fn fetch_registry() -> Result<Registry, String> {
    let urls = build_mirrored_urls(REGISTRY_URL, GITHUB_RAW_MIRRORS);
    let body = fetch_with_fallback(&urls).await?;
    serde_json::from_str(&body).map_err(|e| format!("解析注册表失败: {e}"))
}

fn detect_platform() -> String {
    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "macos"
    };
    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "unknown"
    };
    format!("{os}-{arch}")
}

fn find_enabled_file(dir: &PathBuf, name: &str) -> Option<PathBuf> {
    if !dir.exists() {
        return None;
    }
    for entry in fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        let fname = path.file_name()?.to_string_lossy();
        if fname.ends_with(".disabled") {
            continue;
        }
        if let Some(pname) = extract_plugin_name(&fname) {
            if pname == name {
                return Some(path);
            }
        }
    }
    None
}

fn find_disabled_file(dir: &PathBuf, name: &str) -> Option<PathBuf> {
    if !dir.exists() {
        return None;
    }
    for entry in fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        let fname = path.file_name()?.to_string_lossy();
        if !fname.ends_with(".disabled") {
            continue;
        }
        let base = fname.trim_end_matches(".disabled");
        if let Some(pname) = extract_plugin_name(base) {
            if pname == name {
                return Some(path);
            }
        }
    }
    None
}

fn scan_plugins(plugin_dir: &str) -> Vec<PluginInfo> {
    let dir = PathBuf::from(plugin_dir);
    if !dir.exists() {
        return Vec::new();
    }

    let mut plugins = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = path.file_name().unwrap_or_default().to_string_lossy();

            if let Some(name) = extract_plugin_name(&file_name) {
                plugins.push(PluginInfo {
                    name: name.to_string(),
                    file: file_name.to_string(),
                    enabled: !file_name.ends_with(".disabled"),
                    priority: 0,
                    block_enabled: false,
                    active: false,
                });
            }
        }
    }
    plugins
}

fn extract_plugin_name(file_name: &str) -> Option<&str> {
    if file_name.ends_with(".disabled") {
        let base = file_name.trim_end_matches(".disabled");
        return extract_plugin_name(base);
    }
    if cfg!(target_os = "windows") {
        if file_name.ends_with(".dll") {
            return Some(file_name.trim_end_matches(".dll"));
        }
    } else if file_name.ends_with(".so") {
        let name = file_name.trim_end_matches(".so");
        return Some(name.strip_prefix("lib").unwrap_or(name));
    }
    None
}

// ========== 测试 ==========

#[cfg(test)]
mod tests {
    use super::*;

    // ── build_mirrored_urls ──────────────────────────────────────

    #[test]
    fn test_build_mirrored_urls_registry() {
        let urls = build_mirrored_urls(REGISTRY_URL, GITHUB_RAW_MIRRORS);
        // 原始 URL + 3 个镜像
        assert_eq!(urls.len(), 1 + GITHUB_RAW_MIRRORS.len());
        assert_eq!(urls[0], REGISTRY_URL);
        assert!(urls[1].starts_with("https://ghfast.top/"));
        assert!(urls[1].ends_with("registry.json"));
        assert!(urls[2].starts_with("https://ghproxy.cn/"));
        assert!(urls[3].starts_with("https://raw.gitmirror.com/"));
    }

    #[test]
    fn test_build_mirrored_urls_release() {
        let primary = "https://github.com/luo9-bot/plugin/releases/download/v1.0/plugin.dll";
        let urls = build_mirrored_urls(primary, GITHUB_RELEASE_MIRRORS);
        assert_eq!(urls.len(), 1 + GITHUB_RELEASE_MIRRORS.len());
        assert_eq!(urls[0], primary);
        // 镜像 URL = 前缀 + 原始 URL
        assert_eq!(
            urls[1],
            "https://ghfast.top/https://github.com/luo9-bot/plugin/releases/download/v1.0/plugin.dll"
        );
    }

    #[test]
    fn test_build_mirrored_urls_empty_mirrors() {
        let urls = build_mirrored_urls("https://example.com/file.json", &[]);
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], "https://example.com/file.json");
    }

    // ── detect_platform ──────────────────────────────────────────

    #[test]
    fn test_detect_platform_format() {
        let platform = detect_platform();
        let parts: Vec<&str> = platform.split('-').collect();
        assert_eq!(parts.len(), 2, "平台格式应为 os-arch");
        assert!(
            ["windows", "linux", "macos"].contains(&parts[0]),
            "未知操作系统: {}",
            parts[0]
        );
        assert!(
            ["x86_64", "aarch64", "unknown"].contains(&parts[1]),
            "未知架构: {}",
            parts[1]
        );
    }

    #[test]
    fn test_detect_platform_current() {
        let platform = detect_platform();
        if cfg!(target_os = "windows") {
            assert!(platform.starts_with("windows"));
        } else if cfg!(target_os = "linux") {
            assert!(platform.starts_with("linux"));
        } else {
            assert!(platform.starts_with("macos"));
        }
    }

    // ── extract_plugin_name ──────────────────────────────────────

    #[cfg(target_os = "windows")]
    #[test]
    fn test_extract_plugin_name_dll() {
        assert_eq!(extract_plugin_name("plugin_doro.dll"), Some("plugin_doro"));
        assert_eq!(extract_plugin_name("my_plugin.dll"), Some("my_plugin"));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_extract_plugin_name_disabled_dll() {
        assert_eq!(
            extract_plugin_name("plugin_doro.dll.disabled"),
            Some("plugin_doro")
        );
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_extract_plugin_name_so() {
        assert_eq!(extract_plugin_name("libplugin_doro.so"), Some("plugin_doro"));
        assert_eq!(extract_plugin_name("libmy_plugin.so"), Some("my_plugin"));
        // 没有 lib 前缀的也支持
        assert_eq!(extract_plugin_name("plugin_doro.so"), Some("plugin_doro"));
    }

    #[test]
    fn test_extract_plugin_name_unknown() {
        assert_eq!(extract_plugin_name("readme.txt"), None);
        assert_eq!(extract_plugin_name("config.toml"), None);
    }

    // ── Registry JSON 解析 ───────────────────────────────────────

    #[test]
    fn test_registry_parse_empty() {
        let json = r#"{"plugins": {}}"#;
        let registry: Registry = serde_json::from_str(json).unwrap();
        assert!(registry.plugins.is_empty());
    }

    #[test]
    fn test_registry_parse_single_plugin() {
        let json = r#"{
            "plugins": {
                "test_plugin": {
                    "description": "测试插件",
                    "repo": "luo9-bot/test-plugin",
                    "tags": ["工具"],
                    "versions": [
                        {
                            "version": "1.0.0",
                            "tag": "v1.0.0",
                            "sdk_version": "0.6.0",
                            "assets": {
                                "windows-x86_64": "test_plugin.dll",
                                "linux-x86_64": "libtest_plugin.so"
                            }
                        }
                    ]
                }
            }
        }"#;
        let registry: Registry = serde_json::from_str(json).unwrap();
        let plugin = registry.plugins.get("test_plugin").unwrap();
        assert_eq!(plugin.description, "测试插件");
        assert_eq!(plugin.repo, "luo9-bot/test-plugin");
        assert_eq!(plugin.tags, vec!["工具"]);
        assert_eq!(plugin.versions.len(), 1);
        assert_eq!(plugin.versions[0].version, "1.0.0");
        assert_eq!(plugin.versions[0].assets.len(), 2);
    }

    #[test]
    fn test_registry_parse_no_tags_no_versions() {
        let json = r#"{
            "plugins": {
                "bare_plugin": {
                    "description": "裸插件",
                    "repo": "luo9-bot/bare-plugin"
                }
            }
        }"#;
        let registry: Registry = serde_json::from_str(json).unwrap();
        let plugin = registry.plugins.get("bare_plugin").unwrap();
        assert!(plugin.tags.is_empty());
        assert!(plugin.versions.is_empty());
    }

    // ── URL 构建验证 ─────────────────────────────────────────────

    #[test]
    fn test_registry_url_is_valid() {
        assert!(REGISTRY_URL.starts_with("https://"));
        assert!(REGISTRY_URL.contains("raw.githubusercontent.com"));
        assert!(REGISTRY_URL.ends_with(".json"));
    }

    #[test]
    fn test_mirror_urls_all_https() {
        for mirror in GITHUB_RAW_MIRRORS {
            assert!(mirror.starts_with("https://"), "镜像必须使用 HTTPS: {mirror}");
            assert!(mirror.ends_with('/'), "镜像前缀必须以 / 结尾: {mirror}");
        }
        for mirror in GITHUB_RELEASE_MIRRORS {
            assert!(mirror.starts_with("https://"), "镜像必须使用 HTTPS: {mirror}");
            assert!(mirror.ends_with('/'), "镜像前缀必须以 / 结尾: {mirror}");
        }
    }

    // ── fetch_with_fallback 集成测试 ─────────────────────────────

    #[tokio::test]
    async fn test_fetch_with_fallback_all_fail() {
        let urls = vec![
            "https://this-domain-does-not-exist-12345.com/fail".to_string(),
            "https://this-also-does-not-exist-12345.com/fail".to_string(),
        ];
        let result = fetch_with_fallback(&urls).await;
        assert!(result.is_err(), "所有 URL 都失败时应返回错误");
    }

    #[tokio::test]
    async fn test_fetch_with_fallback_success() {
        // 使用 httpbin 的稳定公共端点
        let urls = vec![
            "https://httpbin.org/get".to_string(),
        ];
        let result = fetch_with_fallback(&urls).await;
        assert!(result.is_ok(), "正常 URL 应成功: {:?}", result.err());
        let body = result.unwrap();
        assert!(body.contains("url"), "响应应包含 url 字段");
    }

    #[tokio::test]
    async fn test_fetch_with_fallback_first_fails_second_succeeds() {
        let urls = vec![
            "https://this-domain-does-not-exist-12345.com/fail".to_string(),
            "https://httpbin.org/get".to_string(),
        ];
        let result = fetch_with_fallback(&urls).await;
        assert!(result.is_ok(), "第一个失败后应 fallback 到第二个: {:?}", result.err());
    }

    #[tokio::test]
    async fn test_fetch_with_fallback_empty_urls() {
        let urls: Vec<String> = vec![];
        let result = fetch_with_fallback(&urls).await;
        assert!(result.is_err(), "空 URL 列表应返回错误");
    }
}
