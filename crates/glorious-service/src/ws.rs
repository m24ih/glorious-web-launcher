//! WebSocket broadcaster for live telemetry.

use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use serde::Serialize;
use tokio::sync::broadcast;

use crate::api::AppState;
use crate::types::DeviceView;

/// Telemetry refresh interval. The dongle needs ~100 ms per feature report, so
/// polling much faster than this would queue transactions.
pub const TELEMETRY_INTERVAL: Duration = Duration::from_secs(3);

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsEvent {
    /// Sent once when a client connects.
    Snapshot { ts: u64, devices: Vec<DeviceView> },
    /// Periodic battery/status refresh.
    Telemetry { ts: u64, devices: Vec<DeviceView> },
    /// A settings patch was applied successfully.
    Applied {
        ts: u64,
        slug: String,
        applied: Vec<String>,
        device: Box<DeviceView>,
    },
    /// Sensor/background failure; the connection stays open.
    Error { ts: u64, message: String },
}

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut rx = state.tx.subscribe();

    // Initial snapshot so a fresh UI renders immediately.
    let manager = state.manager.clone();
    match tokio::task::spawn_blocking(move || manager.snapshot()).await {
        Ok(Ok(devices)) => {
            let event = WsEvent::Snapshot {
                ts: unix_seconds(),
                devices,
            };
            if send_event(&mut socket, &event).await.is_err() {
                return;
            }
        }
        Ok(Err(err)) => {
            let _ = send_event(
                &mut socket,
                &WsEvent::Error {
                    ts: unix_seconds(),
                    message: err.to_string(),
                },
            )
            .await;
        }
        Err(err) => {
            tracing::error!(error = %err, "initial WebSocket snapshot failed");
        }
    }

    loop {
        tokio::select! {
            broadcast = rx.recv() => {
                match broadcast {
                    Ok(event) => {
                        if send_event(&mut socket, &event).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        tracing::warn!(skipped, "WebSocket client lagged behind telemetry");
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            inbound = socket.recv() => {
                match inbound {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Ping(payload))) => {
                        if socket.send(Message::Pong(payload)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Text(text))) => {
                        // Optional manual refresh: {"type":"refresh"}
                        if text.contains("refresh") {
                            let manager = state.manager.clone();
                            if let Ok(Ok(devices)) =
                                tokio::task::spawn_blocking(move || manager.snapshot()).await
                            {
                                let event = WsEvent::Snapshot {
                                    ts: unix_seconds(),
                                    devices,
                                };
                                if send_event(&mut socket, &event).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    Some(Ok(_)) => {}
                    Some(Err(err)) => {
                        tracing::debug!(error = %err, "WebSocket receive error");
                        break;
                    }
                }
            }
        }
    }
}

async fn send_event(socket: &mut WebSocket, event: &WsEvent) -> Result<(), ()> {
    let text = serde_json::to_string(event).map_err(|err| {
        tracing::error!(error = %err, "failed to serialize WebSocket event");
    })?;
    socket
        .send(Message::Text(text.into()))
        .await
        .map_err(|err| {
            tracing::debug!(error = %err, "WebSocket send failed");
        })
}

/// Background task: poll connected devices and broadcast telemetry.
pub fn spawn_telemetry(state: AppState) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(TELEMETRY_INTERVAL);
        // The first tick completes immediately; skip it (clients get a snapshot
        // on connect and the HTTP endpoint is authoritative on startup).
        ticker.tick().await;

        loop {
            ticker.tick().await;

            if state.tx.receiver_count() == 0 {
                continue;
            }

            let manager = state.manager.clone();
            match tokio::task::spawn_blocking(move || manager.snapshot()).await {
                Ok(Ok(devices)) => {
                    let _ = state.tx.send(WsEvent::Telemetry {
                        ts: unix_seconds(),
                        devices,
                    });
                }
                Ok(Err(err)) => {
                    tracing::warn!(error = %err, "telemetry snapshot failed");
                    let _ = state.tx.send(WsEvent::Error {
                        ts: unix_seconds(),
                        message: err.to_string(),
                    });
                }
                Err(err) => {
                    tracing::error!(error = %err, "telemetry task panicked");
                }
            }
        }
    });
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
