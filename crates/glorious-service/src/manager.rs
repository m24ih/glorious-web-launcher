//! Glue between the device database, the HID protocols and the API.
//!
//! All methods are blocking and serialise HID access through `io_lock`: two
//! concurrent feature-report transactions against the same hidraw node can
//! interleave request/response pairs, so the whole operation takes the lock.

use std::collections::HashMap;
use std::fmt;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use hidapi::HidApi;
use serde::Serialize;

use crate::devices::{Connection, DeviceDatabase, DeviceEntry, ProtocolFamily};
use crate::hid::{self, ConnectionKind, DeviceStatus, Endpoint, HidError, SinowealthDevice};
use crate::types::{
    ConnectionView, DeviceStatusView, DeviceView, FirmwareInfo, SettingsPatch, SettingsView,
};

/// One matching product + HID node.
#[derive(Debug, Clone)]
struct Detected {
    entry: DeviceEntry,
    connection: Connection,
    endpoint: Endpoint,
}

/// Result of `POST /api/device/{slug}/apply`.
#[derive(Debug, Clone, Serialize)]
pub struct ApplyResult {
    pub slug: String,
    pub applied: Vec<String>,
    pub device: DeviceView,
}

pub struct DeviceManager {
    db: DeviceDatabase,
    delay: Duration,
    firmware_cache: Mutex<HashMap<String, FirmwareInfo>>,
    io_lock: Mutex<()>,
}

impl DeviceManager {
    pub fn new(db: DeviceDatabase, delay: Duration) -> Self {
        Self {
            db,
            delay,
            firmware_cache: Mutex::new(HashMap::new()),
            io_lock: Mutex::new(()),
        }
    }

    pub fn database(&self) -> &DeviceDatabase {
        &self.db
    }

    /// Connected devices with live battery telemetry. Mirrors
    /// `GET /api/devices`.
    pub fn snapshot(&self) -> Result<Vec<DeviceView>, ManagerError> {
        self.snapshot_impl(false)
    }

    /// Connected devices including DPI/polling/RGB settings. Used by the
    /// `--probe` diagnostic mode.
    pub fn snapshot_detailed(&self) -> Result<Vec<DeviceView>, ManagerError> {
        self.snapshot_impl(true)
    }

    fn snapshot_impl(&self, detailed: bool) -> Result<Vec<DeviceView>, ManagerError> {
        let _guard = self.io_lock.lock().unwrap_or_else(|e| e.into_inner());
        let api = HidApi::new().map_err(|e| ManagerError::Hid(HidError::Backend(e)))?;
        let detected = self.detect(&api);

        Ok(detected
            .iter()
            .map(|device| self.build_view(&api, device, detailed))
            .collect())
    }

    /// One device with full settings. Connected devices are read live; known
    /// but disconnected products return metadata with `connected: false`.
    pub fn device(&self, slug: &str) -> Result<DeviceView, ManagerError> {
        let _guard = self.io_lock.lock().unwrap_or_else(|e| e.into_inner());

        let entries = self.db.find_by_slug(slug);
        if entries.is_empty() {
            return Err(ManagerError::NotFound(slug.to_string()));
        }

        let api = HidApi::new().map_err(|e| ManagerError::Hid(HidError::Backend(e)))?;
        let detected = self.detect(&api);
        if let Some(device) = detected.iter().find(|d| d.entry.slug == slug) {
            return Ok(self.build_view(&api, device, true));
        }

        // Known model, not currently present: return the first record so the UI
        // can still render name, slug and image.
        Ok(disconnected_view(entries[0]))
    }

    /// Apply a settings patch and return a fresh detailed view.
    pub fn apply(&self, slug: &str, patch: &SettingsPatch) -> Result<ApplyResult, ManagerError> {
        patch.validate().map_err(ManagerError::InvalidInput)?;

        let _guard = self.io_lock.lock().unwrap_or_else(|e| e.into_inner());

        let entries = self.db.find_by_slug(slug);
        if entries.is_empty() {
            return Err(ManagerError::NotFound(slug.to_string()));
        }

        let api = HidApi::new().map_err(|e| ManagerError::Hid(HidError::Backend(e)))?;
        let detected = self.detect(&api);
        let device = detected
            .iter()
            .find(|d| d.entry.slug == slug)
            .ok_or_else(|| ManagerError::NotConnected(slug.to_string()))?;

        if device.entry.protocol() != ProtocolFamily::Sinowealth {
            return Err(ManagerError::Unsupported {
                slug: slug.to_string(),
                protocol: device.entry.protocol().as_str().to_string(),
            });
        }

        let handle = SinowealthDevice::open(&api, &device.entry, &device.connection, self.delay)
            .map_err(ManagerError::Hid)?;

        let applied = handle.apply(patch).map_err(ManagerError::Hid)?;

        // Read back so the caller sees what the device actually stored.
        let view = self.build_view(&api, device, true);
        Ok(ApplyResult {
            slug: slug.to_string(),
            applied,
            device: view,
        })
    }

    /// Enumerate HID nodes and map them to database records.
    fn detect(&self, api: &HidApi) -> Vec<Detected> {
        let endpoints = hid::enumerate(api);
        let mut detected = Vec::new();

        for entry in self.db.devices() {
            for connection in &entry.connections {
                let Some((vid, pid)) = connection.vid_pid() else {
                    continue;
                };
                let candidates: Vec<Endpoint> = endpoints
                    .iter()
                    .filter(|e| e.vid == vid && e.pid == pid)
                    .cloned()
                    .collect();
                if candidates.is_empty() {
                    continue;
                }
                let Some(endpoint) = hid::choose_endpoint(&candidates, entry.vendor_usage()) else {
                    continue;
                };
                // First transport listed in the database wins (USB before
                // Dongle when both are plugged in).
                detected.push(Detected {
                    entry: entry.clone(),
                    connection: connection.clone(),
                    endpoint,
                });
                break;
            }
        }

        detected
    }

    fn build_view(&self, api: &HidApi, detected: &Detected, detailed: bool) -> DeviceView {
        let entry = &detected.entry;
        let mut view = base_view(entry, Some(&detected.endpoint), Some(&detected.connection));

        if entry.protocol() != ProtocolFamily::Sinowealth {
            view.supported = false;
            view.status = DeviceStatusView::Unsupported;
            view.error = Some(format!(
                "{} uses the PixArt / Glorious SDK protocol (scaffolded, not implemented)",
                entry.name
            ));
            return view;
        }

        let handle = match SinowealthDevice::open(api, entry, &detected.connection, self.delay) {
            Ok(handle) => handle,
            Err(err) => {
                view.status = DeviceStatusView::Error;
                view.error = Some(err.to_string());
                return view;
            }
        };

        view.firmware = self.firmware_for(&handle, &detected.endpoint);
        view.status = DeviceStatusView::Connected;

        match handle.battery() {
            Ok(battery) => view.battery = Some(battery),
            Err(HidError::Protocol { status, .. })
                if DeviceStatus::from_byte(status) == DeviceStatus::NoDevice =>
            {
                view.status = DeviceStatusView::Asleep;
                view.error = Some("mouse is asleep or out of RF range".into());
            }
            Err(err) => {
                view.status = DeviceStatusView::Error;
                view.error = Some(err.to_string());
            }
        }

        if detailed {
            match handle.settings() {
                Ok(settings) => view.settings = Some(settings),
                Err(HidError::Protocol { status, .. })
                    if DeviceStatus::from_byte(status) == DeviceStatus::NoDevice =>
                {
                    view.status = DeviceStatusView::Asleep;
                    view.error = Some("mouse is asleep or out of RF range".into());
                }
                Err(err) => {
                    if view.error.is_none() {
                        view.error = Some(err.to_string());
                    }
                    view.settings = Some(SettingsView::default());
                }
            }
        }

        view
    }

    fn firmware_for(&self, handle: &SinowealthDevice, endpoint: &Endpoint) -> Option<FirmwareInfo> {
        let key = cache_key(endpoint);

        if let Ok(cache) = self.firmware_cache.lock() {
            if let Some(cached) = cache.get(&key) {
                return Some(cached.clone());
            }
        }

        match handle.firmware() {
            Ok(firmware) => {
                if let Ok(mut cache) = self.firmware_cache.lock() {
                    cache.insert(key, firmware.clone());
                }
                Some(firmware)
            }
            Err(err) => {
                tracing::debug!(endpoint = %endpoint, error = %err, "firmware read failed");
                None
            }
        }
    }
}

fn cache_key(endpoint: &Endpoint) -> String {
    format!(
        "{:04X}:{:04X}:{}",
        endpoint.vid,
        endpoint.pid,
        endpoint.path_display()
    )
}

fn base_view(
    entry: &DeviceEntry,
    endpoint: Option<&Endpoint>,
    connection: Option<&Connection>,
) -> DeviceView {
    let connection_view = match (endpoint, connection) {
        (Some(endpoint), Some(connection)) => Some(ConnectionView {
            kind: connection.kind.clone(),
            vid: format!("0x{:04X}", endpoint.vid),
            pid: format!("0x{:04X}", endpoint.pid),
            interface: endpoint.interface_number,
            usage_page: format!("0x{:04X}", endpoint.usage_page),
            usage: format!("0x{:04X}", endpoint.usage),
            path: endpoint.path_display(),
        }),
        (Some(endpoint), None) => Some(ConnectionView {
            kind: ConnectionKind::Unknown.as_str().to_string(),
            vid: format!("0x{:04X}", endpoint.vid),
            pid: format!("0x{:04X}", endpoint.pid),
            interface: endpoint.interface_number,
            usage_page: format!("0x{:04X}", endpoint.usage_page),
            usage: format!("0x{:04X}", endpoint.usage),
            path: endpoint.path_display(),
        }),
        _ => None,
    };

    DeviceView {
        slug: entry.slug.clone(),
        name: entry.name.clone(),
        id: entry.id.clone(),
        router_id: entry.router_id.clone(),
        protocol: entry.protocol().as_str().to_string(),
        supported: entry.protocol().is_supported(),
        connected: endpoint.is_some(),
        status: if endpoint.is_some() {
            DeviceStatusView::Connected
        } else {
            DeviceStatusView::Disconnected
        },
        connection: connection_view,
        image: entry.image_path().map(str::to_owned),
        battery: None,
        firmware: None,
        settings: None,
        error: None,
        last_seen: endpoint.map(|_| unix_seconds()),
    }
}

fn disconnected_view(entry: &DeviceEntry) -> DeviceView {
    base_view(entry, None, None)
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[derive(Debug)]
pub enum ManagerError {
    NotFound(String),
    NotConnected(String),
    Unsupported { slug: String, protocol: String },
    InvalidInput(String),
    Hid(HidError),
}

impl fmt::Display for ManagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(slug) => write!(f, "unknown device slug '{slug}'"),
            Self::NotConnected(slug) => write!(f, "device '{slug}' is not connected"),
            Self::Unsupported { slug, protocol } => {
                write!(f, "device '{slug}' uses unsupported protocol '{protocol}'")
            }
            Self::InvalidInput(detail) => write!(f, "{detail}"),
            Self::Hid(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for ManagerError {}

impl From<HidError> for ManagerError {
    fn from(value: HidError) -> Self {
        Self::Hid(value)
    }
}

/// Convenience import used by the binary and WebSocket layer.
pub type SharedManager = std::sync::Arc<DeviceManager>;

/// Re-export for callers that only import `manager`.
pub use crate::hid::DeviceStatus as ProtocolStatus;
pub use crate::types::BatteryInfo as Battery;
pub use crate::types::FirmwareInfo as Firmware;

#[cfg(test)]
mod tests {
    use super::*;

    fn manager() -> DeviceManager {
        let (db, _) = DeviceDatabase::load_default().expect("device database");
        DeviceManager::new(db, Duration::from_millis(1))
    }

    #[test]
    fn database_has_model_o_wireless() {
        let manager = manager();
        let entries = manager.database().find_by_slug("model-o-wireless");
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn unknown_slug_is_not_found() {
        let manager = manager();
        match manager.device("does-not-exist") {
            Err(ManagerError::NotFound(slug)) => assert_eq!(slug, "does-not-exist"),
            other => panic!("unexpected result: {other:?}"),
        }
    }

    #[test]
    fn disconnected_device_still_has_metadata() {
        let manager = manager();
        let view = manager.device("model-i-2-wireless").expect("known model");
        assert!(!view.connected);
        assert_eq!(view.status, DeviceStatusView::Disconnected);
        assert_eq!(view.protocol, "pixart");
        assert!(!view.supported);
        assert_eq!(view.name, "Model I 2 Wireless");
    }

    #[test]
    fn cache_key_is_stable() {
        let endpoint = Endpoint {
            vid: 0x258A,
            pid: 0x2022,
            interface_number: 2,
            usage_page: 0xFFFF,
            usage: 0x00,
            product: None,
            manufacturer: None,
            serial: None,
            path: std::ffi::CString::new("/dev/hidraw2").unwrap(),
        };
        assert_eq!(cache_key(&endpoint), "258A:2022:/dev/hidraw2");
    }
}
