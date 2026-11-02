//! Serializable data types shared by the HID layer, the manager and the API.

use serde::{Deserialize, Serialize};

/// Battery telemetry as reported by the mouse.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BatteryInfo {
    /// 0-100. A reported 0 is promoted to 1, matching Glorious CORE.
    pub percentage: u8,
    pub charging: bool,
    /// `ok`, `asleep`, `busy`, `failure`, ...
    pub status: String,
}

/// Firmware versions exposed by the device.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct FirmwareInfo {
    /// 2.4 GHz receiver version (wireless connections only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dongle: Option<String>,
    /// Mouse firmware version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mouse: Option<String>,
    /// Version the UI should show first: dongle when wireless, mouse otherwise.
    pub display: String,
}

/// Live settings read back from the mouse.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SettingsView {
    pub active_profile: u8,
    /// Number of configured DPI stages reported by the firmware.
    pub dpi_stage_count: u8,
    /// DPI values in DPI units. Some Model O Wireless firmware builds report the
    /// stage count but return zeroed resolutions (Glorious CORE has the same
    /// workaround), in which case this stays empty instead of inventing data.
    pub dpi_stages: Vec<u32>,
    /// 1-based stage index, as stored on the device.
    pub active_dpi_stage: u8,
    pub polling_rate: u32,
    pub debounce_ms: u8,
    pub lod: u8,
    pub sleep_seconds: Option<u32>,
}

impl Default for SettingsView {
    fn default() -> Self {
        Self {
            active_profile: 1,
            dpi_stage_count: 0,
            dpi_stages: Vec::new(),
            active_dpi_stage: 1,
            polling_rate: 1000,
            debounce_ms: 10,
            lod: 1,
            sleep_seconds: None,
        }
    }
}

/// Partial update sent to `POST /api/device/{slug}/apply`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettingsPatch {
    /// Profile slot (1-3).
    pub profile: Option<u8>,
    /// DPI stages in DPI units, 1-8 entries.
    pub dpi_stages: Option<Vec<u32>>,
    /// 1-based stage selected by the DPI button.
    pub active_dpi_stage: Option<u8>,
    /// 125, 250, 500, 1000, 2000, 4000 or 8000.
    pub polling_rate: Option<u32>,
    /// 0-50 ms.
    pub debounce_ms: Option<u8>,
    /// 1 or 2 mm.
    pub lod: Option<u8>,
    /// Idle sleep timer in seconds (`0` disables it).
    pub sleep_seconds: Option<u32>,
    pub rgb: Option<RgbPatch>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RgbPatch {
    /// 0-100 %.
    #[serde(default = "default_brightness")]
    pub brightness: u8,
    /// Optional separate wireless brightness (0-100 %); defaults to `brightness`.
    pub wireless_brightness: Option<u8>,
    /// Effect id (0-8) or a name such as `static`, `breathing`, `wave`.
    pub effect: EffectSpec,
    /// Animation rate, 1-100.
    #[serde(default = "default_rate")]
    pub rate: u8,
    /// RGB color, `[r, g, b]`.
    #[serde(default = "default_color")]
    pub color: [u8; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum EffectSpec {
    Id(u8),
    Name(EffectName),
}

/// Effect names accepted by the API. Values match the Sinowealth effect ids.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectName {
    Off,
    Glorious,
    SeamlessBreathing,
    Breathing,
    #[serde(alias = "fixed")]
    Static,
    SingleBreathing,
    Tail,
    Rave,
    Wave,
}

impl EffectName {
    pub fn id(self) -> u8 {
        match self {
            Self::Off => 0,
            Self::Glorious => 1,
            Self::SeamlessBreathing => 2,
            Self::Breathing => 3,
            Self::Static => 4,
            Self::SingleBreathing => 5,
            Self::Tail => 6,
            Self::Rave => 7,
            Self::Wave => 8,
        }
    }
}

impl EffectSpec {
    pub fn resolve(&self) -> Result<u8, String> {
        match self {
            Self::Id(id) if *id <= 8 => Ok(*id),
            Self::Id(id) => Err(format!("rgb.effect id must be 0-8, got {id}")),
            Self::Name(name) => Ok(name.id()),
        }
    }
}

fn default_brightness() -> u8 {
    80
}

fn default_rate() -> u8 {
    50
}

fn default_color() -> [u8; 3] {
    [255, 0, 0]
}

impl SettingsPatch {
    pub fn is_empty(&self) -> bool {
        self.profile.is_none()
            && self.dpi_stages.is_none()
            && self.active_dpi_stage.is_none()
            && self.polling_rate.is_none()
            && self.debounce_ms.is_none()
            && self.lod.is_none()
            && self.sleep_seconds.is_none()
            && self.rgb.is_none()
    }

    /// Validate everything before the first HID write so a bad request cannot
    /// leave the mouse half-configured.
    pub fn validate(&self) -> Result<(), String> {
        if let Some(profile) = self.profile {
            if !(1..=3).contains(&profile) {
                return Err(format!("profile must be 1-3, got {profile}"));
            }
        }

        if let Some(stages) = &self.dpi_stages {
            if stages.is_empty() || stages.len() > 8 {
                return Err(format!(
                    "dpi_stages must contain 1-8 entries, got {}",
                    stages.len()
                ));
            }
            for dpi in stages {
                if !(100..=30000).contains(dpi) {
                    return Err(format!("DPI {dpi} outside the supported 100-30000 range"));
                }
            }
        }

        if let Some(stage) = self.active_dpi_stage {
            if !(1..=8).contains(&stage) {
                return Err(format!("active_dpi_stage must be 1-8, got {stage}"));
            }
            if let Some(stages) = &self.dpi_stages {
                if stage as usize > stages.len() {
                    return Err(format!(
                        "active_dpi_stage {stage} exceeds {} configured stages",
                        stages.len()
                    ));
                }
            }
        }

        if let Some(rate) = self.polling_rate {
            const RATES: [u32; 7] = [125, 250, 500, 1000, 2000, 4000, 8000];
            if !RATES.contains(&rate) {
                return Err(format!("polling_rate must be one of {RATES:?}, got {rate}"));
            }
        }

        if let Some(debounce) = self.debounce_ms {
            if debounce > 50 {
                return Err(format!("debounce_ms must be 0-50, got {debounce}"));
            }
        }

        if let Some(lod) = self.lod {
            if lod != 1 && lod != 2 {
                return Err(format!("lod must be 1 or 2 (mm), got {lod}"));
            }
        }

        if let Some(sleep) = self.sleep_seconds {
            if sleep > u16::MAX as u32 {
                return Err(format!("sleep_seconds must be 0-65535, got {sleep}"));
            }
        }

        if let Some(rgb) = &self.rgb {
            rgb.effect.resolve()?;
            if rgb.brightness > 100 {
                return Err(format!(
                    "rgb.brightness must be 0-100, got {}",
                    rgb.brightness
                ));
            }
            if let Some(brightness) = rgb.wireless_brightness {
                if brightness > 100 {
                    return Err(format!(
                        "rgb.wireless_brightness must be 0-100, got {brightness}"
                    ));
                }
            }
            if !(1..=100).contains(&rgb.rate) {
                return Err(format!("rgb.rate must be 1-100, got {}", rgb.rate));
            }
        }

        Ok(())
    }
}

/// Transport details attached to an API device entry.
#[derive(Debug, Clone, Serialize)]
pub struct ConnectionView {
    #[serde(rename = "type")]
    pub kind: String,
    pub vid: String,
    pub pid: String,
    pub interface: i32,
    pub usage_page: String,
    pub usage: String,
    pub path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceStatusView {
    Connected,
    Asleep,
    Unsupported,
    Error,
    Disconnected,
}

/// API representation of one product, connected or not.
#[derive(Debug, Clone, Serialize)]
pub struct DeviceView {
    pub slug: String,
    pub name: String,
    pub id: String,
    #[serde(rename = "routerID")]
    pub router_id: String,
    pub protocol: String,
    pub supported: bool,
    pub connected: bool,
    pub status: DeviceStatusView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection: Option<ConnectionView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub battery: Option<BatteryInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firmware: Option<FirmwareInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<SettingsView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effect_names_resolve_to_hardware_ids() {
        assert_eq!(EffectSpec::Name(EffectName::Off).resolve(), Ok(0));
        assert_eq!(EffectSpec::Name(EffectName::Static).resolve(), Ok(4));
        assert_eq!(EffectSpec::Name(EffectName::Wave).resolve(), Ok(8));
        assert_eq!(EffectSpec::Id(7).resolve(), Ok(7));
        assert!(EffectSpec::Id(9).resolve().is_err());
    }

    #[test]
    fn effect_names_accept_aliases() {
        let patch: SettingsPatch =
            serde_json::from_str(r#"{"rgb":{"effect":"fixed","brightness":50}}"#).unwrap();
        assert_eq!(patch.rgb.unwrap().effect.resolve(), Ok(4));
    }

    #[test]
    fn validates_dpi_and_polling() {
        let bad = SettingsPatch {
            dpi_stages: Some(vec![50]),
            ..Default::default()
        };
        assert!(bad.validate().is_err());

        let bad = SettingsPatch {
            polling_rate: Some(333),
            ..Default::default()
        };
        assert!(bad.validate().is_err());

        let good = SettingsPatch {
            dpi_stages: Some(vec![800, 1600, 3200]),
            active_dpi_stage: Some(2),
            polling_rate: Some(1000),
            ..Default::default()
        };
        assert!(good.validate().is_ok());
    }

    #[test]
    fn rejects_active_stage_beyond_stage_count() {
        let patch = SettingsPatch {
            dpi_stages: Some(vec![800, 1600]),
            active_dpi_stage: Some(3),
            ..Default::default()
        };
        assert!(patch.validate().is_err());
    }

    #[test]
    fn patch_rejects_unknown_fields() {
        let err = serde_json::from_str::<SettingsPatch>(r#"{"dpi": 800}"#);
        assert!(err.is_err());
    }
}
