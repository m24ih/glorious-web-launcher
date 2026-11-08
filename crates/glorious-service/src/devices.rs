//! Device database extracted from Glorious CORE (`SupportDevice.DB`).
//!
//! The JSON file maps each product to the USB VID/PID pairs it can appear as
//! (wired, 2.4 GHz dongle, Bluetooth) plus the HID usage page/usage pairs that
//! the vendor configuration interface uses. Duplicate slugs are expected: the
//! same physical product can be listed once per firmware generation, each with
//! its own PID set (for example the two `model-o-wireless` entries).

use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Environment variable that overrides device database discovery.
pub const DEVICES_JSON_ENV: &str = "GLORIOUS_DEVICES_JSON";
/// Path of the database relative to the repository root.
pub const DEFAULT_RELATIVE_PATH: &str = "data/glorious_devices.json";

/// Protocol family used by the vendor feature-report interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolFamily {
    /// Sinowealth 65-byte feature report (Model O / Model D series).
    Sinowealth,
    /// PixArt / Glorious SDK family (Model I 2, Model O 2, ...).
    ///
    /// Recognised and listed, but the command set is not implemented yet.
    PixArt,
}

impl ProtocolFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sinowealth => "sinowealth",
            Self::PixArt => "pixart",
        }
    }

    pub fn is_supported(self) -> bool {
        matches!(self, Self::Sinowealth)
    }
}

impl fmt::Display for ProtocolFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A loaded `glorious_devices.json`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeviceDatabase {
    #[serde(default)]
    pub meta: Meta,
    pub devices: Vec<DeviceEntry>,
}

/// Metadata block of the database. Unknown fields are preserved by the source
/// script, so most of them are optional here.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    #[serde(default)]
    pub generated_at: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub total_records: Option<u32>,
    #[serde(default)]
    pub mouse_count: Option<u32>,
    #[serde(default)]
    pub excluded: Vec<Value>,
    #[serde(default)]
    pub images_missing: Vec<Value>,
}

/// One product record.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceEntry {
    pub id: String,
    pub name: String,
    pub slug: String,
    #[serde(rename = "routerID")]
    pub router_id: String,
    #[serde(default)]
    pub model_type: u32,
    #[serde(default)]
    pub image: Option<DeviceImage>,
    #[serde(default)]
    pub connections: Vec<Connection>,
    #[serde(default)]
    pub connection_types: Vec<String>,
    #[serde(default)]
    pub battery: bool,
    #[serde(default)]
    pub layer_max_number: Option<Value>,
    #[serde(default)]
    pub side_light_switch: Option<Value>,
    #[serde(default)]
    pub pairing_flag: Option<Value>,
    #[serde(default)]
    pub fw_update_extension: Vec<String>,
    #[serde(default)]
    pub hid: Option<HidMeta>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeviceImage {
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub file: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
}

/// A physical transport the product can enumerate as.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Connection {
    #[serde(rename = "type")]
    pub kind: String,
    pub vid: String,
    pub pid: String,
}

impl Connection {
    /// Parse the `"0x258A"` strings used in the database.
    pub fn vid_pid(&self) -> Option<(u16, u16)> {
        Some((parse_hex_u16(&self.vid)?, parse_hex_u16(&self.pid)?))
    }

    pub fn vid(&self) -> Option<u16> {
        parse_hex_u16(&self.vid)
    }

    pub fn pid(&self) -> Option<u16> {
        parse_hex_u16(&self.pid)
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct HidMeta {
    #[serde(default)]
    pub set: Vec<HidUsage>,
    #[serde(default)]
    pub get: Vec<HidUsage>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HidUsage {
    pub usagepage: String,
    pub usage: String,
}

impl HidUsage {
    pub fn as_u16(&self) -> Option<(u16, u16)> {
        Some((parse_hex_u16(&self.usagepage)?, parse_hex_u16(&self.usage)?))
    }
}

impl DeviceEntry {
    /// Protocol family inferred from the vendor *set* usage page.
    ///
    /// Sinowealth devices expose the write interface on usage page `0xFFFF`
    /// (Model O/D wired and wireless, including the 65-byte feature report).
    /// Everything else (0xFF00 / 0xFF01) belongs to the PixArt SDK family.
    pub fn protocol(&self) -> ProtocolFamily {
        match self.vendor_usage() {
            Some((0xFFFF, _)) => ProtocolFamily::Sinowealth,
            _ => ProtocolFamily::PixArt,
        }
    }

    /// First HID usage pair the vendor driver writes to (`hid.set[0]`).
    pub fn vendor_usage(&self) -> Option<(u16, u16)> {
        self.hid
            .as_ref()
            .and_then(|h| h.set.first())
            .and_then(HidUsage::as_u16)
    }

    /// Usage pairs the vendor driver reads from (`hid.get`).
    pub fn read_usages(&self) -> Vec<(u16, u16)> {
        self.hid
            .as_ref()
            .map(|h| h.get.iter().filter_map(HidUsage::as_u16).collect())
            .unwrap_or_default()
    }

    /// Newer Sinowealth firmware (Model O 2 Pro generation) uses a different
    /// performance payload layout. Not relevant for the original Model O
    /// Wireless, but kept here for the DPI/polling writers.
    pub fn uses_new_performance_layout(&self) -> bool {
        self.connections.iter().any(|c| {
            matches!(
                c.pid(),
                Some(0x2019 | 0x201A | 0x201B | 0x201C | 0x2033 | 0x2034 | 0x2035 | 0x2036)
            )
        })
    }

    pub fn image_path(&self) -> Option<&str> {
        self.image.as_ref().and_then(|i| i.path.as_deref())
    }

    pub fn connection_of_type(&self, kind: &str) -> Option<&Connection> {
        self.connections
            .iter()
            .find(|c| c.kind.eq_ignore_ascii_case(kind))
    }
}

impl DeviceDatabase {
    /// Load from an explicit file.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, DbError> {
        let path = path.as_ref();
        let raw = std::fs::read_to_string(path)
            .map_err(|e| DbError::Io(path.to_path_buf(), e.to_string()))?;
        let db: Self = serde_json::from_str(&raw)
            .map_err(|e| DbError::Parse(path.to_path_buf(), e.to_string()))?;
        if db.devices.is_empty() {
            return Err(DbError::Empty(path.to_path_buf()));
        }
        Ok(db)
    }

    /// Resolve the database using, in order:
    ///
    /// 1. `GLORIOUS_DEVICES_JSON`
    /// 2. `./data/glorious_devices.json` (current directory)
    /// 3. `<workspace>/data/glorious_devices.json` relative to this crate
    /// 4. `<executable dir>/../data/...` and `<executable dir>/data/...`
    ///
    /// Returns the database and the file it was loaded from.
    pub fn load_default() -> Result<(Self, PathBuf), DbError> {
        let candidates = default_candidates();
        let mut tried = Vec::new();

        for candidate in &candidates {
            if !candidate.is_file() {
                tried.push(candidate.display().to_string());
                continue;
            }
            match Self::from_path(candidate) {
                Ok(db) => return Ok((db, candidate.clone())),
                Err(err) => tried.push(format!("{err}")),
            }
        }

        Err(DbError::NotFound(tried))
    }

    pub fn devices(&self) -> &[DeviceEntry] {
        &self.devices
    }

    /// Every record carrying `slug` (duplicates are expected across firmware
    /// generations).
    pub fn find_by_slug(&self, slug: &str) -> Vec<&DeviceEntry> {
        self.devices.iter().filter(|d| d.slug == slug).collect()
    }

    /// Resolve a VID/PID pair to a product record.
    pub fn find_by_vid_pid(&self, vid: u16, pid: u16) -> Option<&DeviceEntry> {
        self.devices.iter().find(|device| {
            device
                .connections
                .iter()
                .any(|c| c.vid() == Some(vid) && c.pid() == Some(pid))
        })
    }

    /// Resolve the connection descriptor matching a VID/PID pair.
    pub fn find_connection(&self, vid: u16, pid: u16) -> Option<(&DeviceEntry, &Connection)> {
        self.devices.iter().find_map(|device| {
            device
                .connections
                .iter()
                .find(|c| c.vid() == Some(vid) && c.pid() == Some(pid))
                .map(|conn| (device, conn))
        })
    }
}

/// Candidate locations for automatic database discovery.
pub fn default_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(path) = std::env::var_os(DEVICES_JSON_ENV) {
        candidates.push(PathBuf::from(path));
    }

    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join(DEFAULT_RELATIVE_PATH));
    }

    // `env!("CARGO_MANIFEST_DIR")` = <repo>/crates/glorious-service
    candidates.push(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(DEFAULT_RELATIVE_PATH),
    );

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("../..").join(DEFAULT_RELATIVE_PATH));
            candidates.push(dir.join("..").join(DEFAULT_RELATIVE_PATH));
            candidates.push(dir.join("data").join("glorious_devices.json"));
            candidates.push(dir.join("..").join("data").join("glorious_devices.json"));
        }
    }

    candidates.dedup();
    candidates
}

/// Parse `"0x258A"`, `"258a"` or `"258A"` into an integer.
pub fn parse_hex_u16(input: &str) -> Option<u16> {
    let trimmed = input.trim();
    let without_prefix = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    u16::from_str_radix(without_prefix, 16).ok()
}

#[derive(Debug)]
pub enum DbError {
    Io(PathBuf, String),
    Parse(PathBuf, String),
    Empty(PathBuf),
    NotFound(Vec<String>),
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(path, err) => write!(f, "failed to read {}: {err}", path.display()),
            Self::Parse(path, err) => write!(f, "invalid JSON in {}: {err}", path.display()),
            Self::Empty(path) => write!(f, "{} contains no devices", path.display()),
            Self::NotFound(tried) => {
                write!(
                    f,
                    "glorious_devices.json not found. Tried: {}",
                    tried.join(", ")
                )
            }
        }
    }
}

impl std::error::Error for DbError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_db() -> DeviceDatabase {
        let (db, path) = DeviceDatabase::load_default().expect("repository device database");
        assert!(path.ends_with(DEFAULT_RELATIVE_PATH));
        db
    }

    #[test]
    fn loads_extracted_database() {
        let db = repo_db();
        assert!(db.devices.len() >= 16);
        assert_eq!(db.meta.total_records, Some(26));
    }

    #[test]
    fn resolves_model_o_wireless_dongle_pid() {
        let db = repo_db();
        let (entry, connection) = db
            .find_connection(0x258A, 0x2022)
            .expect("Model O Wireless dongle PID");
        assert_eq!(entry.slug, "model-o-wireless");
        assert_eq!(entry.name, "MODEL O WIRELESS");
        assert_eq!(connection.kind, "Dongle");
        assert_eq!(entry.protocol(), ProtocolFamily::Sinowealth);
        assert_eq!(entry.vendor_usage(), Some((0xFFFF, 0x0000)));
    }

    #[test]
    fn classifies_model_i_2_as_pixart_scaffold() {
        let db = repo_db();
        let entry = db
            .find_by_vid_pid(0x093A, 0x821A)
            .expect("Model I 2 Wireless");
        assert_eq!(entry.slug, "model-i-2-wireless");
        assert_eq!(entry.protocol(), ProtocolFamily::PixArt);
        assert!(!entry.protocol().is_supported());
    }

    #[test]
    fn duplicate_slugs_are_kept() {
        let db = repo_db();
        let entries = db.find_by_slug("model-o-wireless");
        assert_eq!(entries.len(), 2);
        assert!(entries
            .iter()
            .any(|e| e.connection_of_type("Dongle").is_some()));
    }

    #[test]
    fn parses_hex_strings() {
        assert_eq!(parse_hex_u16("0x258A"), Some(0x258A));
        assert_eq!(parse_hex_u16("258a"), Some(0x258A));
        assert_eq!(parse_hex_u16("821A"), Some(0x821A));
        assert_eq!(parse_hex_u16("nope"), None);
    }
}
