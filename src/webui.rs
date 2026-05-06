use axum::{
    Router,
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::info;

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
}

#[derive(Serialize, Deserialize)]
struct PluginInfo {
    name: String,
    file: String,
    enabled: bool,
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

pub async fn start(host: &str, port: u16, plugin_dir: String) {
    let state = Arc::new(WebuiState {
        plugin_dir,
        start_time: std::time::Instant::now(),
    });

    let app = Router::new()
        .route("/", get(index_page))
        .route("/style.css", get(style_css))
        .route("/app.js", get(app_js))
        .route("/api/status", get(api_status))
        .route("/api/plugins", get(api_plugins))
        .route("/api/plugins/upload", post(api_plugin_upload))
        .route("/api/plugins/{name}", delete(api_plugin_delete))
        .route("/api/plugins/{name}/enable", post(api_plugin_enable))
        .route("/api/plugins/{name}/disable", post(api_plugin_disable))
        .route("/api/plugins/install/{name}", post(api_plugin_install))
        .route("/api/registry", get(api_registry))
        .route("/api/logs", get(api_logs))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("{}:{}", host, port);
    info!("WebUI 启动于 http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

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
    let plugins = scan_plugins(&state.plugin_dir);
    Json(plugins)
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

    if let Err(e) = fs::rename(&enabled_path, &disabled_path) {
        return json_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("禁用失败: {e}"));
    }

    info!("插件已禁用: {name}");
    json_ok(&format!("插件 {name} 已禁用，重启后生效"))
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
