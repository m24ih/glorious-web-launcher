//! Axum HTTP API and shared application state.

use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use axum::extract::{Path, State};
use axum::http::StatusCode;
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
}

impl AppState {
    pub fn new(manager: Arc<DeviceManager>) -> Self {
        let (tx, _rx) = broadcast::channel(64);
        Self {
            manager,
            tx,
            started_at: Instant::now(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DeviceList {
    pub count: usize,
    pub timestamp: u64,
    pub devices: Vec<DeviceView>,
}

/// Build the HTTP/WebSocket router.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/devices", get(list_devices))
        .route("/api/device/{slug}", get(device_details))
        .route("/api/device/{slug}/apply", post(apply_device))
        .route("/ws", get(ws::ws_handler))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn health(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": env!("CARGO_PKG_NAME"),
        "version": env!("CARGO_PKG_VERSION"),
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
}
