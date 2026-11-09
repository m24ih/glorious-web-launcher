//! Axum HTTP API and shared application state.

use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use axum::extract::{Path, State};
use axum::http::{header, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;
use serde_json::{json, Value};
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;

use crate::manager::{ApplyResult, DeviceManager, ManagerError, SharedManager};
use crate::types::{DeviceView, SettingsPatch};
use crate::ws::{self, WsEvent};

/// Default bind address. The service is local-only by design.
pub const DEFAULT_BIND: &str = "127.0.0.1:25820";

#[derive(Clone)]
pub struct AppState {
    pub manager: SharedManager,
    pub tx: broadcast::Sender<WsEvent>,
    pub started_at: Instant,
    pub tray_settings: Arc<crate::tray_settings::TraySettingsStore>,
}

impl AppState {
    pub fn new(manager: Arc<DeviceManager>) -> Self {
        let (tx, _rx) = broadcast::channel(64);
        Self {
            manager,
            tx,
            started_at: Instant::now(),
            tray_settings: Arc::new(crate::tray_settings::TraySettingsStore::load_default()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DeviceList {
    pub count: usize,
    pub timestamp: u64,
    pub devices: Vec<DeviceView>,
}

#[derive(rust_embed::RustEmbed)]
#[folder = "../../frontend/dist"]
pub struct EmbeddedAssets;

/// Fallback handler serving embedded single-page application static files.
async fn static_handler(uri: Uri) -> Response {
    let raw_path = uri.path().trim_start_matches('/');
    let path = if raw_path.is_empty() {
        "index.html"
    } else {
        raw_path
    };

    if let Some(file) = EmbeddedAssets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return (
            StatusCode::OK,
            [(header::CONTENT_TYPE, mime.as_ref())],
            file.data,
        )
            .into_response();
    }

    if path == "api" || path.starts_with("api/") {
        return (StatusCode::NOT_FOUND, Json(json!({ "error": "not found" }))).into_response();
    }

    // SPA fallback: return index.html for non-API client routes
    if let Some(index) = EmbeddedAssets::get("index.html") {
        let mime = mime_guess::from_path("index.html").first_or_octet_stream();
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, mime.as_ref())],
            index.data,
        )
            .into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "index.html not found" })),
        )
            .into_response()
    }
}

/// Build the HTTP/WebSocket router.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route(
            "/api/tray-settings",
            get(get_tray_settings).put(put_tray_settings),
        )
        .route("/api/devices", get(list_devices))
        .route("/api/device/{slug}", get(device_details))
        .route("/api/device/{slug}/apply", post(apply_device))
        .route("/api/update/install", post(update_install))
        .route("/ws", get(ws::ws_handler))
        .fallback(static_handler)
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn get_tray_settings(
    State(state): State<AppState>,
) -> Json<crate::tray_settings::TraySettings> {
    Json(state.tray_settings.get())
}

async fn put_tray_settings(
    State(state): State<AppState>,
    Json(settings): Json<crate::tray_settings::TraySettings>,
) -> Result<Json<crate::tray_settings::TraySettings>, ApiError> {
    let store = state.tray_settings.clone();
    let saved = settings.clone();
    tokio::task::spawn_blocking(move || store.set(saved))
        .await
        .map_err(ApiError::join)?
        .map_err(|error| ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("Could not save tray settings: {error}"),
        })?;
    Ok(Json(settings))
}

async fn health(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": env!("CARGO_PKG_NAME"),
        "version": env!("CARGO_PKG_VERSION"),
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "uptime_seconds": state.started_at.elapsed().as_secs(),
        "ws_subscribers": state.tx.receiver_count(),
    }))
}

async fn list_devices(State(state): State<AppState>) -> Result<Json<DeviceList>, ApiError> {
    let manager = state.manager.clone();
    let devices = tokio::task::spawn_blocking(move || manager.snapshot())
        .await
        .map_err(ApiError::join)??;

    Ok(Json(DeviceList {
        count: devices.len(),
        timestamp: unix_seconds(),
        devices,
    }))
}

async fn device_details(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<DeviceView>, ApiError> {
    let manager = state.manager.clone();
    let device = tokio::task::spawn_blocking(move || manager.device(&slug))
        .await
        .map_err(ApiError::join)??;
    Ok(Json(device))
}

async fn apply_device(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(patch): Json<SettingsPatch>,
) -> Result<Json<ApplyResult>, ApiError> {
    if patch.is_empty() {
        return Err(ApiError::bad_request(
            "request body must contain at least one setting",
        ));
    }

    let manager = state.manager.clone();
    let slug_for_task = slug.clone();
    let result = tokio::task::spawn_blocking(move || manager.apply(&slug_for_task, &patch))
        .await
        .map_err(ApiError::join)??;

    let _ = state.tx.send(WsEvent::Applied {
        ts: unix_seconds(),
        slug: result.slug.clone(),
        applied: result.applied.clone(),
        device: Box::new(result.device.clone()),
    });

    Ok(Json(result))
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateInstallRequest {
    pub download_url: String,
    pub version: String,
}

async fn update_install(Json(req): Json<UpdateInstallRequest>) -> Response {
    let url = req.download_url.trim();
    if !url.starts_with("https://github.com/m24ih/glorious-web-launcher/releases/")
        && !url.starts_with("https://gitlab.com/m24ih/glorious-web-launcher/-/releases/")
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid download URL source" })),
        )
            .into_response();
    }

    let safe_version: String = req
        .version
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-')
        .collect();
    let safe_version = if safe_version.is_empty() {
        "latest".to_string()
    } else {
        safe_version
    };

    #[cfg(target_os = "linux")]
    if cfg!(target_os = "linux") {
        match install_linux_update(url, &safe_version).await {
            Ok(()) => {
                return (
                    StatusCode::OK,
                    Json(json!({
                        "status": "installing",
                        "message": "Update applied. Glorious Web Launcher is restarting..."
                    })),
                )
                    .into_response();
            }
            Err(error) => {
                tracing::error!(%error, "failed to install Linux update");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": error })),
                )
                    .into_response();
            }
        }
    }

    let setup_path =
        std::env::temp_dir().join(format!("GloriousWebLauncher-Setup-v{safe_version}.exe"));

    let curl_bin = if cfg!(windows) { "curl.exe" } else { "curl" };
    let setup_str = match setup_path.to_str() {
        Some(s) => s,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid temporary path" })),
            )
                .into_response();
        }
    };

    let status = match tokio::process::Command::new(curl_bin)
        .args(["-fSL", url, "-o", setup_str])
        .status()
        .await
    {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(error = %e, "failed to run curl for update");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to download update binary" })),
            )
                .into_response();
        }
    };

    if !status.success() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to download update binary" })),
        )
            .into_response();
    }

    #[cfg(windows)]
    {
        if let Err(e) = std::process::Command::new(&setup_path)
            .args(["/SILENT", "/CLOSEAPPLICATIONS"])
            .spawn()
        {
            tracing::error!(error = %e, "failed to launch installer");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to launch installer" })),
            )
                .into_response();
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "status": "installing",
            "message": "Update downloaded. The installer will now update Glorious Web Launcher."
        })),
    )
        .into_response()
}

#[cfg(target_os = "linux")]
async fn install_linux_update(url: &str, safe_version: &str) -> Result<(), String> {
    use std::fs::{self, File};
    use std::os::unix::fs::PermissionsExt;

    let current_exe = std::env::current_exe().map_err(|error| error.to_string())?;
    let current_dir = current_exe
        .parent()
        .ok_or_else(|| "Could not determine current executable directory".to_string())?;
    let temp_dir = std::env::temp_dir();
    let file_name = format!(
        "glorious-service-update-{safe_version}-{}",
        std::process::id()
    );
    let (download_path, install_dir) = match File::create(current_dir.join(&file_name)) {
        Ok(_) => (current_dir.join(&file_name), current_dir.to_path_buf()),
        Err(_) => (temp_dir.join(&file_name), temp_dir.clone()),
    };

    let download_str = download_path
        .to_str()
        .ok_or_else(|| "Invalid temporary path".to_string())?;
    let status = tokio::process::Command::new("curl")
        .args(["-fSL", url, "-o", download_str])
        .status()
        .await
        .map_err(|error| format!("Failed to run curl: {error}"))?;
    if !status.success() {
        let _ = fs::remove_file(&download_path);
        return Err("Failed to download update binary".to_string());
    }

    let new_bin_path = if url.to_ascii_lowercase().contains(".tar.gz") {
        let extracted_path =
            install_dir.join(format!("glorious-service-update-{}", std::process::id()));
        let archive = File::open(&download_path).map_err(|error| error.to_string())?;
        let decoder = flate2::read::GzDecoder::new(archive);
        let mut archive = tar::Archive::new(decoder);
        let mut found = false;
        for entry in archive.entries().map_err(|error| error.to_string())? {
            let mut entry = entry.map_err(|error| error.to_string())?;
            let path = entry
                .path()
                .map_err(|error| error.to_string())?
                .into_owned();
            if path.file_name().and_then(|name| name.to_str()) == Some("glorious-service") {
                let mut output =
                    File::create(&extracted_path).map_err(|error| error.to_string())?;
                std::io::copy(&mut entry, &mut output).map_err(|error| error.to_string())?;
                found = true;
                break;
            }
        }
        if !found {
            let _ = fs::remove_file(&download_path);
            return Err("Update archive does not contain glorious-service".to_string());
        }
        extracted_path
    } else {
        download_path.clone()
    };

    fs::set_permissions(&new_bin_path, fs::Permissions::from_mode(0o755))
        .map_err(|error| error.to_string())?;
    fs::rename(&new_bin_path, &current_exe).map_err(|error| error.to_string())?;
    let _ = fs::remove_file(&download_path);

    let restarted = tokio::process::Command::new("systemctl")
        .args(["--user", "restart", "glorious-launcher.service"])
        .status()
        .await
        .map(|status| status.success())
        .unwrap_or(false);
    if !restarted {
        let mut command = std::process::Command::new(&current_exe);
        command.args(std::env::args_os().skip(1));
        command
            .spawn()
            .map_err(|error| format!("Failed to restart updated service: {error}"))?;
    }
    Ok(())
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Error response: `{"error": "..."}` with a meaningful HTTP status.
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    fn join(err: tokio::task::JoinError) -> Self {
        tracing::error!(error = %err, "blocking task failed");
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "background task failed".to_string(),
        }
    }
}

impl From<ManagerError> for ApiError {
    fn from(err: ManagerError) -> Self {
        let status = match &err {
            ManagerError::NotFound(_) => StatusCode::NOT_FOUND,
            ManagerError::NotConnected(_) => StatusCode::CONFLICT,
            ManagerError::Unsupported { .. } => StatusCode::NOT_IMPLEMENTED,
            ManagerError::InvalidInput(_) => StatusCode::BAD_REQUEST,
            ManagerError::Hid(_) => StatusCode::BAD_GATEWAY,
        };
        Self {
            status,
            message: err.to_string(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(json!({ "error": self.message }))).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_bind_is_loopback_only() {
        assert!(DEFAULT_BIND.starts_with("127.0.0.1:"));
        assert!(DEFAULT_BIND.ends_with(":25820"));
    }

    #[test]
    fn manager_errors_map_to_http_statuses() {
        let cases = [
            (ManagerError::NotFound("x".into()), StatusCode::NOT_FOUND),
            (ManagerError::NotConnected("x".into()), StatusCode::CONFLICT),
            (
                ManagerError::Unsupported {
                    slug: "x".into(),
                    protocol: "pixart".into(),
                },
                StatusCode::NOT_IMPLEMENTED,
            ),
            (
                ManagerError::InvalidInput("bad".into()),
                StatusCode::BAD_REQUEST,
            ),
        ];
        for (err, expected) in cases {
            assert_eq!(ApiError::from(err).status, expected);
        }
    }

    fn test_router() -> Router {
        let (db, _) = crate::devices::DeviceDatabase::load_default().expect("device db");
        let manager = Arc::new(DeviceManager::new(db, std::time::Duration::from_millis(1)));
        let state = AppState::new(manager);
        router(state)
    }

    #[tokio::test]
    async fn static_root_serves_index_html() {
        use axum::body::Body;
        use axum::http::Request;
        use http_body_util::BodyExt;
        use tower::ServiceExt;

        let app = test_router();
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/html"
        );
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body_str = std::str::from_utf8(&body).unwrap();
        assert!(body_str.contains("<html") || body_str.contains("<!DOCTYPE html>"));
    }

    #[tokio::test]
    async fn static_spa_fallback_serves_index_html() {
        use axum::body::Body;
        use axum::http::Request;
        use tower::ServiceExt;

        let app = test_router();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/devices/model-o-wireless")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/html"
        );
    }

    #[tokio::test]
    async fn static_known_asset_served_with_correct_mime() {
        use axum::body::Body;
        use axum::http::Request;
        use tower::ServiceExt;

        let app = test_router();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/favicon.svg")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "image/svg+xml"
        );
    }

    #[tokio::test]
    async fn unknown_api_route_returns_404_json() {
        use axum::body::Body;
        use axum::http::Request;
        use http_body_util::BodyExt;
        use tower::ServiceExt;

        let app = test_router();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/unknown_endpoint")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"], "not found");
    }

    #[tokio::test]
    async fn update_install_rejects_invalid_url() {
        use axum::body::Body;
        use axum::http::Request;
        use http_body_util::BodyExt;
        use tower::ServiceExt;

        let app = test_router();
        let payload = serde_json::json!({
            "download_url": "https://malicious-site.com/malware.exe",
            "version": "0.1.4"
        });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/update/install")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"], "Invalid download URL source");
    }
}
