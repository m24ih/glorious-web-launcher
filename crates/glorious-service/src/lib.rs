//! Glorious Web Launcher hardware service.
//!
//! The crate is split into four layers:
//!
//! * [`devices`] loads `data/glorious_devices.json` and maps VID/PID pairs to
//!   Glorious product metadata and protocol families.
//! * [`hid`] talks to the vendor HID feature-report interface. The Sinowealth
//!   (Model O / D series) implementation is complete; the PixArt family used by
//!   Model I 2 / Model O 2 is scaffolded.
//! * [`manager`] combines both into snapshots that the Axum API and WebSocket
//!   broadcaster expose.
//! * [`api`] / [`ws`] serve the HTTP and WebSocket surface.
//!
//! All HID calls are blocking, so handlers run them on `spawn_blocking`.

pub mod api;
pub mod devices;
pub mod hid;
pub mod manager;
pub mod profiles;
pub mod types;
pub mod ws;

pub use devices::{DeviceDatabase, DeviceEntry};
pub use manager::DeviceManager;
pub use profiles::ProfileStore;
pub use types::{
    BatteryInfo, ConnectionView, DeviceStatusView, DeviceView, EffectName, EffectSpec,
    FirmwareInfo, LightingView, RgbPatch, SettingsPatch, SettingsView,
};
