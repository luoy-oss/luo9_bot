use axum::{
    Router,
    extract::{Multipart, Path, Query, Request, State},
    http::{StatusCode, header},
    middleware::{self, Next},
    response::{Html, IntoResponse, Json, Response},
    routing::{delete, get, post, put},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

use crate::utils::logger;

const REGISTRY_URL: &str = "https://raw.githubusercontent.com/luo9-bot/registry/main/registry.json";

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

pub struct WebuiState {
    pub plugin_dir: String,
    pub start_time: std::time::Instant,
    pub token: String,
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

    let state = Arc::new(WebuiState {
        plugin_dir,
        start_time: std::time::Instant::now(),
        token: token.clone(),
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
                // 已安装但不知道具体版本，显示文件名
                installed_map
                    .get(name.as_str())
                    .map(|p| p.file.clone())
                    .unwrap_or_default()
            }),
        });
    }

    // 按名称排序
    available.sort_by(|a, b| a.name.cmp(&b.name));
    Json(available).into_response()
}

/// 从注册表安装插件
async fn api_plugin_install(
    State(state): State<Arc<WebuiState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let registry = match fetch_registry().await {
        Ok(r) => r,
        Err(e) => return json_err(StatusCode::BAD_GATEWAY, &format!("获取注册表失败: {e}")),
    };

    let plugin = match registry.plugins.get(&name) {
        Some(p) => p,
        None => return json_err(StatusCode::NOT_FOUND, &format!("插件 {name} 不在注册表中")),
    };

    let latest = match plugin.versions.first() {
        Some(v) => v,
        None => return json_err(StatusCode::NOT_FOUND, &format!("插件 {name} 没有可用版本")),
    };

    // 确定平台和资源文件名
    let platform_key = detect_platform();
    let asset_name = match latest.assets.get(&platform_key) {
        Some(a) => a.clone(),
        None => {
            return json_err(
                StatusCode::BAD_REQUEST,
                &format!("插件 {name} 不支持当前平台 {platform_key}"),
            )
        }
    };

    // 构建下载 URL
    let download_url = format!(
        "https://github.com/{}/releases/download/{}/{}",
        plugin.repo, latest.tag, asset_name
    );

    info!("正在下载插件: {} v{} ({})", name, latest.version, download_url);

    // 下载文件
    let client = reqwest::Client::new();
    let resp = match client
        .get(&download_url)
        .header("User-Agent", "luo9-bot")
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return json_err(StatusCode::BAD_GATEWAY, &format!("下载请求失败: {e}")),
    };

    if !resp.status().is_success() {
        return json_err(
            StatusCode::BAD_GATEWAY,
            &format!("下载失败，HTTP {}", resp.status()),
        );
    }

    let bytes = match resp.bytes().await {
        Ok(b) => b,
        Err(e) => return json_err(StatusCode::BAD_GATEWAY, &format!("下载读取失败: {e}")),
    };

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

    info!("插件安装成功: {} v{} -> {}", name, latest.version, target.display());
    json_ok(&format!(
        "插件 {} v{} 安装成功，重启后生效",
        name, latest.version
    ))
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

    if let Err(e) = fs::rename(&disabled_path, &enabled_path) {
        return json_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("启用失败: {e}"));
    }

    info!("插件已启用: {name}");
    json_ok(&format!("插件 {name} 已启用，重启后生效"))
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

    // 运行时禁用（取消订阅，等待线程退出）
    {
        let mut manager = crate::plugin::GLOBAL_PLUGIN_MANAGER.lock().await;
        match manager.disable_plugin(&name).await {
            Ok(msg) => info!("{}", msg),
            Err(e) => warn!("运行时禁用插件 {} 失败: {}", name, e),
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
    let mut manager = crate::plugin::GLOBAL_PLUGIN_MANAGER.lock().await;
    match manager.disable_plugin(&name).await {
        Ok(_) => {}
        Err(e) => {
            // 如果插件已经是禁用状态，仍然尝试重新加载
            if !e.contains("已经是禁用状态") {
                return json_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("禁用失败: {e}"));
            }
        }
    }
    // TODO: 实现重新加载逻辑（需要 loader 支持单个插件重新加载）
    json_ok(&format!("插件 {name} 已禁用，需要重启才能重新加载"))
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

async fn fetch_registry() -> Result<Registry, String> {
    let client = reqwest::Client::new();
    let body = client
        .get(REGISTRY_URL)
        .header("User-Agent", "luo9-bot")
        .send()
        .await
        .map_err(|e| format!("请求失败: {e}"))?
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {e}"))?;

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
