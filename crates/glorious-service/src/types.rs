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

/// Lighting configuration for a profile.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LightingView {
    pub effect: u8,
    pub effect_name: String,
    pub brightness: u8,
    pub rate: u8,
    pub color: [u8; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum ButtonAction {
    Mouse {
        button: u8,
    },
    Keyboard {
        key_code: u8,
        #[serde(default)]
        modifiers: u8,
    },
    Dpi {
        action: u8,
    },
    Profile {
        action: u8,
    },
    Media {
        key: u16,
    },
    Macro {
        macro_id: u16,
        #[serde(default)]
        repeat_mode: u8,
        #[serde(default = "default_macro_loop_count")]
        loop_count: u8,
    },
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ButtonBinding {
    pub pin: String,
    pub button_id: u8,
    pub action: ButtonAction,
}

/// A single macro event record on the device.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MacroEventDto {
    pub keydown: bool,
    pub key: String,
    pub times: u32,
}

/// One configured macro stored on the device.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MacroItem {
    pub id: u16,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub repeat_mode: u8,
    #[serde(default = "default_macro_loop_count")]
    pub loop_count: u8,
    pub raw_bytes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MacroDataDto {
    pub id: u16,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub repeat_mode: u8,
    #[serde(default = "default_macro_loop_count")]
    pub loop_count: u8,
    #[serde(default)]
    pub events: Vec<MacroEventDto>,
    #[serde(default)]
    pub raw_bytes: Option<Vec<u8>>,
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
    /// DPI stage colors as hex strings (e.g. `["#FFFF00", "#0000FF", ...]`).
    #[serde(default)]
    pub dpi_colors: Vec<String>,
    /// 1-based stage index, as stored on the device.
    pub active_dpi_stage: u8,
    pub polling_rate: u32,
    pub debounce_ms: u8,
    pub lod: u8,
    pub sleep_seconds: Option<u32>,
    #[serde(default)]
    pub lighting: Option<LightingView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buttons: Option<Vec<ButtonBinding>>,
}

impl Default for SettingsView {
    fn default() -> Self {
        Self {
            active_profile: 1,
            dpi_stage_count: 0,
            dpi_stages: Vec::new(),
            dpi_colors: Vec::new(),
            active_dpi_stage: 1,
            polling_rate: 1000,
            debounce_ms: 10,
            lod: 1,
            sleep_seconds: None,
            lighting: None,
            buttons: None,
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
    /// DPI stage colors as hex strings (e.g. `["#FFFF00", ...]`), 1-8 entries.
    pub dpi_colors: Option<Vec<String>>,
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
    #[serde(alias = "lighting")]
    pub rgb: Option<RgbPatch>,
    #[serde(default)]
    pub buttons: Option<Vec<ButtonBinding>>,
    #[serde(default)]
    pub macros: Option<Vec<MacroItem>>,
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
    /// Optional effect name.
    #[serde(default)]
    pub effect_name: Option<String>,
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

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Glorious => "glorious",
            Self::SeamlessBreathing => "seamless_breathing",
            Self::Breathing => "breathing",
            Self::Static => "static",
            Self::SingleBreathing => "single_breathing",
            Self::Tail => "tail",
            Self::Rave => "rave",
            Self::Wave => "wave",
        }
    }

    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(Self::Off),
            1 => Some(Self::Glorious),
            2 => Some(Self::SeamlessBreathing),
            3 => Some(Self::Breathing),
            4 => Some(Self::Static),
            5 => Some(Self::SingleBreathing),
            6 => Some(Self::Tail),
            7 => Some(Self::Rave),
            8 => Some(Self::Wave),
            _ => None,
        }
    }
}

pub fn effect_name_for(id: u8) -> String {
    EffectName::from_id(id)
        .map(|e| e.as_str().to_string())
        .unwrap_or_else(|| format!("effect_{id}"))
}

pub fn parse_hex_color(hex: &str) -> Option<[u8; 3]> {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
    let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
    let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
    Some([r, g, b])
}

pub fn normalize_hex_color(hex: &str) -> String {
    let clean = hex.trim().trim_start_matches('#');
    format!("#{}", clean.to_ascii_uppercase())
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

fn default_macro_loop_count() -> u8 {
    1
}

fn validate_button_binding(binding: &ButtonBinding) -> Result<(), String> {
    match &binding.action {
        ButtonAction::Mouse { button } => {
            let valid = (1..=8).contains(button)
                || *button == 0x10
                || *button == 0x11
                || *button == 0x18
                || *button == 0x19
                || *button == 0x0C;
            if !valid {
                return Err(format!("invalid mouse button {button}"));
            }
        }
        ButtonAction::Profile { action } => {
            if *action == 0 || *action > 4 {
                return Err(format!("invalid profile action {action}"));
            }
        }
        ButtonAction::Keyboard { key_code, .. } => {
            if *key_code == 0 {
                return Err("keyboard key_code cannot be 0".to_string());
            }
        }
        ButtonAction::Dpi { action } => {
            if *action == 0 {
                return Err("dpi action cannot be 0".to_string());
            }
        }
        ButtonAction::Media { key } => {
            if *key == 0 {
                return Err("media key cannot be 0".to_string());
            }
        }
        ButtonAction::Macro { macro_id, repeat_mode, loop_count } => {
            if *macro_id == 0 {
                return Err(format!(
                    "buttons[{}].action.macro_id must be at least 1",
                    binding.button_id
                ));
            }
            if *repeat_mode > 2 {
                return Err(format!(
                    "buttons[{}].action.repeat_mode must be 0 (no repeat), 1 (toggle), or 2 (while holding)",
                    binding.button_id
                ));
            }
            if *loop_count == 0 {
                return Err(format!(
                    "buttons[{}].action.loop_count must be at least 1",
                    binding.button_id
                ));
            }
        }
        ButtonAction::Disabled => {}
    }
    Ok(())
}

impl SettingsPatch {
    pub fn is_empty(&self) -> bool {
        self.profile.is_none()
            && self.dpi_stages.is_none()
            && self.dpi_colors.is_none()
            && self.active_dpi_stage.is_none()
            && self.polling_rate.is_none()
            && self.debounce_ms.is_none()
            && self.lod.is_none()
            && self.sleep_seconds.is_none()
            && self.rgb.is_none()
            && self.buttons.is_none()
            && self.macros.is_none()
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

        if let Some(colors) = &self.dpi_colors {
            if colors.is_empty() || colors.len() > 8 {
                return Err(format!(
                    "dpi_colors must contain 1-8 entries, got {}",
                    colors.len()
                ));
            }
            for c in colors {
                if parse_hex_color(c).is_none() {
                    return Err(format!("invalid hex color '{c}' in dpi_colors"));
                }
            }
            if let Some(stages) = &self.dpi_stages {
                if stages.len() != colors.len() {
                    return Err(format!(
                        "dpi_stages length ({}) does not match dpi_colors length ({})",
                        stages.len(),
                        colors.len()
                    ));
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

        if let Some(buttons) = &self.buttons {
            for b in buttons {
                if b.button_id == 0 || b.button_id > 32 {
                    return Err(format!("button_id must be 1-32, got {}", b.button_id));
                }
                validate_button_binding(b)?;
            }
        }

        if let Some(macros) = &self.macros {
            for m in macros {
                if m.id < 1 {
                    return Err(format!("macro.id must be at least 1, got {}", m.id));
                }
                if m.repeat_mode > 2 {
                    return Err(format!(
                        "macro[{}].repeat_mode must be 0-2, got {}",
                        m.id, m.repeat_mode
                    ));
                }
                if m.raw_bytes.len() > 4096 {
                    return Err(format!(
                        "macro[{}].raw_bytes length exceeds 4096 bytes (got {})",
                        m.id,
                        m.raw_bytes.len()
                    ));
                }
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
    pub profiles: Option<std::collections::BTreeMap<u8, SettingsView>>,
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

    #[test]
    fn parses_and_normalizes_hex_colors() {
        assert_eq!(parse_hex_color("#FFFF00"), Some([255, 255, 0]));
        assert_eq!(parse_hex_color("0000FF"), Some([0, 0, 255]));
        assert_eq!(parse_hex_color("#xyz123"), None);
        assert_eq!(parse_hex_color("#123"), None);
        assert_eq!(normalize_hex_color("ffff00"), "#FFFF00");
        assert_eq!(normalize_hex_color("#00ff00"), "#00FF00");
    }

    #[test]
    fn validates_dpi_colors() {
        let good = SettingsPatch {
            dpi_stages: Some(vec![400, 800]),
            dpi_colors: Some(vec!["#FFFF00".into(), "#0000FF".into()]),
            ..Default::default()
        };
        assert!(good.validate().is_ok());

        let bad_color = SettingsPatch {
            dpi_colors: Some(vec!["not-a-color".into()]),
            ..Default::default()
        };
        assert!(bad_color.validate().is_err());

        let mismatch = SettingsPatch {
            dpi_stages: Some(vec![400, 800]),
            dpi_colors: Some(vec!["#FFFF00".into()]),
            ..Default::default()
        };
        assert!(mismatch.validate().is_err());
    }

    #[test]
    fn deserializes_button_bindings() {
        let json = r#"{
            "buttons": [
                {"pin": "left", "button_id": 1, "action": {"type": "mouse", "button": 1}},
                {"pin": "dpi", "button_id": 6, "action": {"type": "dpi", "action": 1}},
                {"pin": "forward", "button_id": 4, "action": {"type": "keyboard", "key_code": 4, "modifiers": 2}},
                {"pin": "back", "button_id": 5, "action": {"type": "media", "key": 233}},
                {"pin": "middle", "button_id": 3, "action": {"type": "disabled"}}
            ]
        }"#;
        let patch: SettingsPatch = serde_json::from_str(json).expect("valid button json");
        let buttons = patch.buttons.as_ref().expect("buttons present");
        assert_eq!(buttons.len(), 5);
        assert_eq!(buttons[0].button_id, 1);
        assert_eq!(buttons[0].action, ButtonAction::Mouse { button: 1 });
        assert_eq!(
            buttons[2].action,
            ButtonAction::Keyboard {
                key_code: 4,
                modifiers: 2
            }
        );
        assert_eq!(buttons[4].action, ButtonAction::Disabled);
        assert!(patch.validate().is_ok());
    }

    #[test]
    fn validates_button_bindings() {
        let bad_button_id = SettingsPatch {
            buttons: Some(vec![ButtonBinding {
                pin: "btn".into(),
                button_id: 0,
                action: ButtonAction::Disabled,
            }]),
            ..Default::default()
        };
        assert!(bad_button_id.validate().is_err());

        let bad_mouse = SettingsPatch {
            buttons: Some(vec![ButtonBinding {
                pin: "btn".into(),
                button_id: 1,
                action: ButtonAction::Mouse { button: 0 },
            }]),
            ..Default::default()
        };
        assert!(bad_mouse.validate().is_err());

        let bad_key = SettingsPatch {
            buttons: Some(vec![ButtonBinding {
                pin: "btn".into(),
                button_id: 1,
                action: ButtonAction::Keyboard {
                    key_code: 0,
                    modifiers: 0,
                },
            }]),
            ..Default::default()
        };
        assert!(bad_key.validate().is_err());

        for accepted_id in [16, 17, 20, 32] {
            let good = SettingsPatch {
                buttons: Some(vec![ButtonBinding {
                    pin: "btn".into(),
                    button_id: accepted_id,
                    action: ButtonAction::Disabled,
                }]),
                ..Default::default()
            };
            assert!(good.validate().is_ok(), "button_id {accepted_id} should be accepted");
        }

        let bad_button_33 = SettingsPatch {
            buttons: Some(vec![ButtonBinding {
                pin: "btn".into(),
                button_id: 33,
                action: ButtonAction::Disabled,
            }]),
            ..Default::default()
        };
        assert!(bad_button_33.validate().is_err());

        for accepted_mouse in [0x10, 0x11, 0x18, 0x19, 0x0C] {
            let good_mouse = SettingsPatch {
                buttons: Some(vec![ButtonBinding {
                    pin: "btn".into(),
                    button_id: 1,
                    action: ButtonAction::Mouse { button: accepted_mouse },
                }]),
                ..Default::default()
            };
            assert!(good_mouse.validate().is_ok(), "mouse button 0x{:02X} should be accepted", accepted_mouse);
        }

        let bad_mouse_9 = SettingsPatch {
            buttons: Some(vec![ButtonBinding {
                pin: "btn".into(),
                button_id: 1,
                action: ButtonAction::Mouse { button: 9 },
            }]),
            ..Default::default()
        };
        assert!(bad_mouse_9.validate().is_err());

        for action in 1..=4 {
            let good_profile = SettingsPatch {
                buttons: Some(vec![ButtonBinding {
                    pin: "profile_btn".into(),
                    button_id: 1,
                    action: ButtonAction::Profile { action },
                }]),
                ..Default::default()
            };
            assert!(good_profile.validate().is_ok(), "profile action {action} should be valid");
        }

        for bad_action in [0, 5] {
            let bad_profile = SettingsPatch {
                buttons: Some(vec![ButtonBinding {
                    pin: "profile_btn".into(),
                    button_id: 1,
                    action: ButtonAction::Profile { action: bad_action },
                }]),
                ..Default::default()
            };
            assert!(bad_profile.validate().is_err(), "profile action {bad_action} should be invalid");
        }
    }

    #[test]
    fn deserializes_macro_button_action() {
        let json = r#"{"pin":"m","button_id":2,"action":{"type":"macro","macro_id":5,"repeat_mode":1,"loop_count":3}}"#;
        let binding: ButtonBinding = serde_json::from_str(json).unwrap();
        assert_eq!(binding.button_id, 2);
        assert!(matches!(binding.action, ButtonAction::Macro { macro_id: 5, repeat_mode: 1, loop_count: 3 }));
    }

    #[test]
    fn macro_defaults_to_loop_count_1() {
        let json = r#"{"pin":"m","button_id":3,"action":{"type":"macro","macro_id":10}}"#;
        let binding: ButtonBinding = serde_json::from_str(json).unwrap();
        match binding.action {
            ButtonAction::Macro { macro_id, repeat_mode, loop_count } => {
                assert_eq!(macro_id, 10);
                assert_eq!(repeat_mode, 0); // default
                assert_eq!(loop_count, 1);   // default
            }
            _ => panic!("expected Macro"),
        }
    }

    #[test]
    fn validates_macro_button_bindings() {
        // valid
        let good = SettingsPatch {
            buttons: Some(vec![ButtonBinding {
                pin: "m".into(),
                button_id: 2,
                action: ButtonAction::Macro { macro_id: 3, repeat_mode: 2, loop_count: 5 },
            }]),
            ..Default::default()
        };
        assert!(good.validate().is_ok());

        // macro_id == 0
        let bad_id = SettingsPatch {
            buttons: Some(vec![ButtonBinding {
                pin: "m".into(),
                button_id: 1,
                action: ButtonAction::Macro { macro_id: 0, repeat_mode: 0, loop_count: 1 },
            }]),
            ..Default::default()
        };
        assert!(bad_id.validate().is_err());

        // repeat_mode > 2
        let bad_repeat = SettingsPatch {
            buttons: Some(vec![ButtonBinding {
                pin: "m".into(),
                button_id: 1,
                action: ButtonAction::Macro { macro_id: 2, repeat_mode: 3, loop_count: 1 },
            }]),
            ..Default::default()
        };
        assert!(bad_repeat.validate().is_err());

        // loop_count == 0
        let bad_loop = SettingsPatch {
            buttons: Some(vec![ButtonBinding {
                pin: "m".into(),
                button_id: 1,
                action: ButtonAction::Macro { macro_id: 2, repeat_mode: 1, loop_count: 0 },
            }]),
            ..Default::default()
        };
        assert!(bad_loop.validate().is_err());
    }

    #[test]
    fn validates_settings_patch_macros() {
        let good = SettingsPatch {
            macros: Some(vec![MacroItem {
                id: 1,
                name: "Test Macro".to_string(),
                repeat_mode: 1,
                loop_count: 3,
                raw_bytes: vec![0x02, 0x04, 0x03, 0x04],
            }]),
            ..Default::default()
        };
        assert!(good.validate().is_ok());

        // id == 0
        let bad_id = SettingsPatch {
            macros: Some(vec![MacroItem {
                id: 0,
                name: "Bad ID".to_string(),
                repeat_mode: 0,
                loop_count: 1,
                raw_bytes: vec![],
            }]),
            ..Default::default()
        };
        assert!(bad_id.validate().is_err());

        // repeat_mode > 2
        let bad_repeat = SettingsPatch {
            macros: Some(vec![MacroItem {
                id: 1,
                name: "Bad Repeat".to_string(),
                repeat_mode: 3,
                loop_count: 1,
                raw_bytes: vec![],
            }]),
            ..Default::default()
        };
        assert!(bad_repeat.validate().is_err());

        // raw_bytes > 4096
        let bad_len = SettingsPatch {
            macros: Some(vec![MacroItem {
                id: 1,
                name: "Huge Macro".to_string(),
                repeat_mode: 0,
                loop_count: 1,
                raw_bytes: vec![0; 4097],
            }]),
            ..Default::default()
        };
        assert!(bad_len.validate().is_err());
    }
}
