//! Software profile store and persistence for Glorious devices.
//!
//! Because Sinowealth hardware does not expose read commands for RGB effects or
//! DPI stage colors, profiles are saved in `~/.config/glorious-launcher/profiles.json`.
//! If that file does not exist, profiles are automatically imported from existing
//! Glorious CORE installations (`DeviceDB.db`).

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::types::{
    effect_name_for, normalize_hex_color, ButtonBinding, LightingView, MacroItem, SettingsPatch,
    SettingsView,
};

/// Configuration for one profile slot (1-based, 1 to 3).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileConfig {
    pub name: String,
    pub lighting: LightingView,
    pub dpi_stages: Vec<u32>,
    pub dpi_colors: Vec<String>,
    pub active_dpi_stage: u8,
    pub polling_rate: u32,
    pub debounce_ms: u8,
    pub lod: u8,
    pub sleep_seconds: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buttons: Option<Vec<ButtonBinding>>,
}

/// Alias for stored profile settings.
pub type StoredSettings = ProfileConfig;

impl ProfileConfig {
    pub fn to_settings_view(&self, active_profile: u8) -> SettingsView {
        SettingsView {
            active_profile,
            dpi_stage_count: self.dpi_stages.len() as u8,
            dpi_stages: self.dpi_stages.clone(),
            dpi_colors: self.dpi_colors.clone(),
            active_dpi_stage: self.active_dpi_stage,
            polling_rate: self.polling_rate,
            debounce_ms: self.debounce_ms,
            lod: self.lod,
            sleep_seconds: self.sleep_seconds,
            lighting: Some(self.lighting.clone()),
            buttons: self.buttons.clone(),
        }
    }
}

/// Profiles and active slot for a single product slug.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceProfileConfig {
    pub active_profile: u8,
    pub profiles: BTreeMap<u8, ProfileConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub macros: Option<Vec<MacroItem>>,
}

/// Root JSON structure saved to `profiles.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileStoreData {
    pub version: u32,
    pub devices: BTreeMap<String, DeviceProfileConfig>,
}

impl Default for ProfileStoreData {
    fn default() -> Self {
        Self {
            version: 1,
            devices: BTreeMap::new(),
        }
    }
}

/// Thread-safe profile store managing persistence.
pub struct ProfileStore {
    path: PathBuf,
    data: Mutex<ProfileStoreData>,
}

impl ProfileStore {
    /// Standard path: `~/.config/glorious-launcher/profiles.json` (Linux) or
    /// `%APPDATA%\glorious-launcher\profiles.json` (Windows).
    pub fn default_path() -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            if let Some(appdata) = std::env::var_os("APPDATA") {
                return PathBuf::from(appdata)
                    .join("glorious-launcher")
                    .join("profiles.json");
            }
        }
        if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
            PathBuf::from(xdg).join("glorious-launcher").join("profiles.json")
        } else if let Some(home) = std::env::var_os("HOME") {
            PathBuf::from(home)
                .join(".config")
                .join("glorious-launcher")
                .join("profiles.json")
        } else if let Some(userprofile) = std::env::var_os("USERPROFILE") {
            PathBuf::from(userprofile)
                .join(".config")
                .join("glorious-launcher")
                .join("profiles.json")
        } else {
            PathBuf::from(".config")
                .join("glorious-launcher")
                .join("profiles.json")
        }
    }

    /// Load default `profiles.json`, auto-importing from Glorious CORE or defaults.
    pub fn load_default() -> Self {
        Self::load(Self::default_path())
    }

    /// In-memory store without disk operations, useful for unit tests.
    pub fn in_memory(data: ProfileStoreData) -> Self {
        Self {
            path: PathBuf::from("/dev/null"),
            data: Mutex::new(data),
        }
    }

    /// Load from a specific file path.
    pub fn load(path: PathBuf) -> Self {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(data) = serde_json::from_str::<ProfileStoreData>(&content) {
                    return Self {
                        path,
                        data: Mutex::new(data),
                    };
                }
            }
        }

        // Auto-import from Glorious CORE if available, otherwise fall back to defaults.
        let data = if let Some(gcore_path) = find_glorious_core_db() {
            match import_glorious_core_db(&gcore_path) {
                Ok(imported) => {
                    tracing::info!(
                        source = %gcore_path.display(),
                        target = %path.display(),
                        "auto-imported profiles from Glorious CORE"
                    );
                    imported
                }
                Err(err) => {
                    tracing::warn!(error = %err, "failed to import Glorious CORE DB, using defaults");
                    Self::default_data()
                }
            }
        } else {
            Self::default_data()
        };

        let store = Self {
            path,
            data: Mutex::new(data),
        };
        let _ = store.save();
        store
    }

    pub fn default_data() -> ProfileStoreData {
        let mut devices = BTreeMap::new();
        devices.insert(
            "model-o-wireless".to_string(),
            sensible_default_device("model-o-wireless"),
        );
        ProfileStoreData {
            version: 1,
            devices,
        }
    }

    /// Persist current in-memory state to `self.path`.
    pub fn save(&self) -> Result<(), io::Error> {
        let data = self.data.lock().unwrap_or_else(|e| e.into_inner());
        self.save_locked(&data)
    }

    fn save_locked(&self, data: &ProfileStoreData) -> Result<(), io::Error> {
        if self.path == Path::new("/dev/null") {
            return Ok(());
        }
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(data)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        fs::write(&self.path, json)
    }

    /// Get active profile id for a device (1-based, defaults to 1).
    pub fn active_profile(&self, slug: &str) -> u8 {
        let data = self.data.lock().unwrap_or_else(|e| e.into_inner());
        data.devices
            .get(slug)
            .map(|d| d.active_profile)
            .unwrap_or(1)
    }

    /// Settings view for a specific profile slot.
    pub fn get_settings_view(&self, slug: &str, profile_id: u8) -> SettingsView {
        let data = self.data.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(dev) = data.devices.get(slug) {
            if let Some(p) = dev.profiles.get(&profile_id) {
                return p.to_settings_view(profile_id);
            }
            if let Some(p) = dev.profiles.get(&dev.active_profile) {
                return p.to_settings_view(dev.active_profile);
            }
            if let Some((&id, p)) = dev.profiles.iter().next() {
                return p.to_settings_view(id);
            }
        }
        sensible_default_profile(profile_id).to_settings_view(profile_id)
    }

    /// Settings view for currently active profile.
    pub fn get_current_settings_view(&self, slug: &str) -> SettingsView {
        let active = self.active_profile(slug);
        self.get_settings_view(slug, active)
    }

    /// Apply a validated patch and save to disk.
    pub fn update(&self, slug: &str, patch: &SettingsPatch) -> Result<(), io::Error> {
        let mut data = self.data.lock().unwrap_or_else(|e| e.into_inner());
        let dev = data
            .devices
            .entry(slug.to_string())
            .or_insert_with(|| sensible_default_device(slug));

        if let Some(requested_p) = patch.profile {
            dev.active_profile = requested_p;
        }
        let profile_id = dev.active_profile;

        let profile = dev
            .profiles
            .entry(profile_id)
            .or_insert_with(|| sensible_default_profile(profile_id));

        if let Some(stages) = &patch.dpi_stages {
            profile.dpi_stages = stages.clone();
        }
        if let Some(colors) = &patch.dpi_colors {
            profile.dpi_colors = colors.iter().map(|c| normalize_hex_color(c)).collect();
        }
        if let Some(active_stage) = patch.active_dpi_stage {
            profile.active_dpi_stage = active_stage;
        }
        if let Some(rate) = patch.polling_rate {
            profile.polling_rate = rate;
        }
        if let Some(debounce) = patch.debounce_ms {
            profile.debounce_ms = debounce;
        }
        if let Some(lod) = patch.lod {
            profile.lod = lod;
        }
        if let Some(sleep) = patch.sleep_seconds {
            profile.sleep_seconds = if sleep == 0 { None } else { Some(sleep) };
        }
        if let Some(rgb) = &patch.rgb {
            let effect_id = rgb.effect.resolve().unwrap_or(0);
            let effect_name = effect_name_for(effect_id);
            profile.lighting = LightingView {
                effect: effect_id,
                effect_name,
                brightness: rgb.brightness,
                rate: rgb.rate,
                color: rgb.color,
            };
        }
        if let Some(buttons) = &patch.buttons {
            profile.buttons = Some(buttons.clone());
        }
        if let Some(macros) = &patch.macros {
            dev.macros = Some(macros.clone());
        }

        self.save_locked(&data)
    }

    /// Synchronize live values read from hardware (active profile, DPI stage, polling, etc.).
    /// Persists to disk if profile slot or active DPI stage changed.
    pub fn sync_hardware_telemetry(
        &self,
        slug: &str,
        active_profile: u8,
        active_dpi_stage: u8,
        polling_rate: u32,
        debounce_ms: u8,
        lod: u8,
        sleep_seconds: Option<u32>,
    ) {
        let mut data = self.data.lock().unwrap_or_else(|e| e.into_inner());
        let dev = data
            .devices
            .entry(slug.to_string())
            .or_insert_with(|| sensible_default_device(slug));

        let changed = dev.active_profile != active_profile;
        dev.active_profile = active_profile;

        let profile = dev
            .profiles
            .entry(active_profile)
            .or_insert_with(|| sensible_default_profile(active_profile));

        let stage_changed = profile.active_dpi_stage != active_dpi_stage;
        profile.active_dpi_stage = active_dpi_stage;
        profile.polling_rate = polling_rate;
        profile.debounce_ms = debounce_ms;
        profile.lod = lod;
        if sleep_seconds.is_some() {
            profile.sleep_seconds = sleep_seconds;
        }

        if changed || stage_changed {
            let _ = self.save_locked(&data);
        }
    }
}

/// Fallback profile values.
pub fn sensible_default_profile(profile_id: u8) -> ProfileConfig {
    ProfileConfig {
        name: format!("PROFILE{profile_id}"),
        lighting: LightingView {
            effect: 1,
            effect_name: "glorious".to_string(),
            brightness: 80,
            rate: 50,
            color: [255, 0, 0],
        },
        dpi_stages: vec![400, 800, 1600, 3200],
        dpi_colors: vec![
            "#FFFF00".to_string(),
            "#0000FF".to_string(),
            "#FF0000".to_string(),
            "#00FF00".to_string(),
        ],
        active_dpi_stage: 1,
        polling_rate: 1000,
        debounce_ms: 10,
        lod: 1,
        sleep_seconds: Some(600),
        buttons: None,
    }
}

/// Fallback device values with 3 profiles.
pub fn sensible_default_device(_slug: &str) -> DeviceProfileConfig {
    let mut profiles = BTreeMap::new();
    for id in 1..=3 {
        profiles.insert(id, sensible_default_profile(id));
    }
    DeviceProfileConfig {
        active_profile: 1,
        profiles,
        macros: None,
    }
}

/// Search for existing Glorious CORE database (`DeviceDB.db`).
pub fn find_glorious_core_db() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        for env_var in &["APPDATA", "LOCALAPPDATA"] {
            if let Some(base) = std::env::var_os(env_var) {
                let userdata = PathBuf::from(base).join("Glorious Core").join("userdata");
                if let Ok(entries) = fs::read_dir(&userdata) {
                    for entry in entries.flatten() {
                        let path = entry.path().join("data").join("DeviceDB.db");
                        if path.exists() {
                            return Some(path);
                        }
                    }
                }
            }
        }
    }

    // 1. Explicit path
    let explicit = PathBuf::from("/home/melih/.config/Glorious Core/userdata/melih/data/DeviceDB.db");
    if explicit.exists() {
        return Some(explicit);
    }

    // 2. Generic ~/.config/Glorious Core/userdata/*/data/DeviceDB.db
    let config_dir = if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        PathBuf::from(xdg)
    } else if let Some(home) = std::env::var_os("HOME") {
        PathBuf::from(home).join(".config")
    } else {
        return None;
    };

    let userdata = config_dir.join("Glorious Core").join("userdata");
    if let Ok(entries) = fs::read_dir(&userdata) {
        for entry in entries.flatten() {
            let path = entry.path().join("data").join("DeviceDB.db");
            if path.exists() {
                return Some(path);
            }
        }
    }

    None
}

/// Parse and import Glorious CORE database from file.
pub fn import_glorious_core_db(path: &Path) -> Result<ProfileStoreData, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    import_glorious_core_db_from_str(&content)
}

/// Parse Glorious CORE database lines into `ProfileStoreData`.
pub fn import_glorious_core_db_from_str(content: &str) -> Result<ProfileStoreData, String> {
    let mut store_data = ProfileStoreData::default();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let record: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(e) => {
                tracing::debug!("skipping malformed DeviceDB line: {e}");
                continue;
            }
        };

        let devicename = record
            .get("devicename")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let slug = if !devicename.is_empty() {
            devicename
                .to_lowercase()
                .replace(' ', "-")
                .replace('_', "-")
        } else {
            "model-o-wireless".to_string()
        };

        let active_profile = record
            .get("profileindex")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as u8;
        let sleep_enabled = record
            .get("sleep")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let sleep_time_mins = record
            .get("sleeptime")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as u32;
        let sleep_seconds = if sleep_enabled {
            Some(sleep_time_mins * 60)
        } else {
            None
        };

        let mut profiles = BTreeMap::new();

        if let Some(profile_array) = record.get("profile").and_then(|v| v.as_array()) {
            for (idx, p) in profile_array.iter().enumerate() {
                let profile_id = p
                    .get("profileid")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(idx as u64 + 1) as u8;
                let name = p
                    .get("profileName")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&format!("PROFILE{profile_id}"))
                    .to_string();

                let perf = p.get("performance");
                let polling_rate = perf
                    .and_then(|p| p.get("pollingrate"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1000) as u32;
                let debounce_ms = perf
                    .and_then(|p| p.get("DebounceValue"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(10) as u8;
                let lod = perf
                    .and_then(|p| p.get("LodValue"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1) as u8;
                let active_dpi_stage = perf
                    .and_then(|p| p.get("dpiSelectIndex"))
                    .and_then(|v| v.as_u64())
                    .map(|idx| idx as u8 + 1)
                    .unwrap_or(1);

                let mut dpi_stages = Vec::new();
                let mut dpi_colors = Vec::new();
                if let Some(stages) = perf.and_then(|p| p.get("DpiStage")).and_then(|v| v.as_array()) {
                    for s in stages {
                        if let Some(val) = s.get("value").and_then(|v| v.as_u64()) {
                            dpi_stages.push(val as u32);
                        }
                        if let Some(c) = s.get("color").and_then(|v| v.as_str()) {
                            dpi_colors.push(normalize_hex_color(c));
                        }
                    }
                }
                if dpi_stages.is_empty() {
                    dpi_stages = vec![400, 800, 1600, 3200];
                }
                if dpi_colors.is_empty() {
                    dpi_colors = vec![
                        "#FFFF00".to_string(),
                        "#0000FF".to_string(),
                        "#FF0000".to_string(),
                        "#00FF00".to_string(),
                    ];
                }

                let light = p.get("lighting");
                let effect = light
                    .and_then(|l| l.get("Effect"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u8;
                let effect_name = effect_name_for(effect);
                let rate = light
                    .and_then(|l| l.get("RateValue"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(60) as u8;
                let brightness = light
                    .and_then(|l| {
                        l.get("WirelessBrightnessValue")
                            .or_else(|| l.get("WiredBrightnessValue"))
                            .or_else(|| l.get("OpacityValue"))
                    })
                    .and_then(|v| v.as_u64())
                    .unwrap_or(60) as u8;

                let mut color = [255, 0, 0];
                if let Some(colors) = light.and_then(|l| l.get("Color")).and_then(|v| v.as_array()) {
                    if let Some(first) = colors.first() {
                        let r = first.get("R").and_then(|v| v.as_u64()).unwrap_or(255) as u8;
                        let g = first.get("G").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
                        let b = first.get("B").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
                        color = [r, g, b];
                    }
                }

                let lighting = LightingView {
                    effect,
                    effect_name,
                    brightness,
                    rate,
                    color,
                };

                profiles.insert(
                    profile_id,
                    ProfileConfig {
                        name,
                        lighting,
                        dpi_stages,
                        dpi_colors,
                        active_dpi_stage,
                        polling_rate,
                        debounce_ms,
                        lod,
                        sleep_seconds,
                        buttons: None,
                    },
                );
            }
        }

        if profiles.is_empty() {
            profiles.insert(1, sensible_default_profile(1));
        }

        store_data.devices.insert(
            slug,
            DeviceProfileConfig {
                active_profile,
                profiles,
                macros: None,
            },
        );
    }

    Ok(store_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ButtonAction;

    const SAMPLE_DEVICEDB_LINE: &str = r#"{"vid":["0x258A","0x258A"],"pid":["0x2011","0x2022"],"SN":"0x258A0x2011","devicename":"MODEL O WIRELESS","ModelType":1,"image":"ModelO","battery":true,"profile":[{"profileName":"PROFILE1","profileid":1,"performance":{"pollingrate":1000,"inputLatency":16,"LodValue":1,"DebounceValue":12,"DpiNum":4,"dpiSelectIndex":3,"DpiStage":[{"value":400,"color":"FFFF00"},{"value":800,"color":"0000FF"},{"value":1600,"color":"FF0000"},{"value":3200,"color":"00FF00"}]},"lighting":{"Effect":0,"RateValue":60,"OpacityValue":60,"WiredBrightnessValue":60,"WirelessBrightnessValue":60,"SepatateCheckValue":false,"Color":[{"R":255,"G":0,"B":0,"flag":true}]}},{"profileName":"PROFILE2","profileid":2,"performance":{"pollingrate":1000,"inputLatency":16,"LodValue":1,"DebounceValue":10,"DpiNum":4,"dpiSelectIndex":2,"DpiStage":[{"value":400,"color":"FFFF00"},{"value":800,"color":"0000FF"},{"value":1600,"color":"FF0000"},{"value":3200,"color":"00FF00"}]},"lighting":{"Effect":0,"RateValue":60,"OpacityValue":60,"WiredBrightnessValue":60,"WirelessBrightnessValue":60,"SepatateCheckValue":false,"Color":[{"R":255,"G":0,"B":0,"flag":true}]}}],"profileindex":1,"value":2,"_id":"j5L9StqKTiuDNECV","sleep":true,"sleeptime":10}"#;

    #[test]
    fn imports_from_glorious_core_db_text() {
        let store = import_glorious_core_db_from_str(SAMPLE_DEVICEDB_LINE).expect("parsed");
        assert!(store.devices.contains_key("model-o-wireless"));
        let dev = &store.devices["model-o-wireless"];
        assert_eq!(dev.active_profile, 1);
        assert_eq!(dev.profiles.len(), 2);

        let p1 = &dev.profiles[&1];
        assert_eq!(p1.name, "PROFILE1");
        assert_eq!(p1.polling_rate, 1000);
        assert_eq!(p1.debounce_ms, 12);
        assert_eq!(p1.lod, 1);
        assert_eq!(p1.active_dpi_stage, 4); // dpiSelectIndex 3 -> 4
        assert_eq!(p1.dpi_stages, vec![400, 800, 1600, 3200]);
        assert_eq!(
            p1.dpi_colors,
            vec!["#FFFF00", "#0000FF", "#FF0000", "#00FF00"]
        );
        assert_eq!(p1.sleep_seconds, Some(600)); // 10 mins * 60
        assert_eq!(p1.lighting.effect, 0);
        assert_eq!(p1.lighting.effect_name, "off");
        assert_eq!(p1.lighting.brightness, 60);
        assert_eq!(p1.lighting.rate, 60);
        assert_eq!(p1.lighting.color, [255, 0, 0]);

        let p2 = &dev.profiles[&2];
        assert_eq!(p2.debounce_ms, 10);
        assert_eq!(p2.active_dpi_stage, 3); // dpiSelectIndex 2 -> 3
    }

    #[test]
    fn falls_back_to_defaults() {
        let default_data = ProfileStore::default_data();
        let store = ProfileStore::in_memory(default_data);
        let view = store.get_current_settings_view("model-o-wireless");
        assert_eq!(view.active_profile, 1);
        assert_eq!(view.polling_rate, 1000);
        assert_eq!(view.debounce_ms, 10);
        assert_eq!(view.lod, 1);
        assert_eq!(view.active_dpi_stage, 1);
        assert_eq!(view.dpi_stages, vec![400, 800, 1600, 3200]);
        assert_eq!(
            view.dpi_colors,
            vec!["#FFFF00", "#0000FF", "#FF0000", "#00FF00"]
        );
        assert!(view.lighting.is_some());
    }

    #[test]
    fn updates_via_patch_in_memory() {
        let store = ProfileStore::in_memory(ProfileStore::default_data());
        let patch = SettingsPatch {
            debounce_ms: Some(4),
            lod: Some(2),
            active_dpi_stage: Some(3),
            dpi_stages: Some(vec![800, 1600, 3200]),
            dpi_colors: Some(vec![
                "#112233".into(),
                "#445566".into(),
                "#778899".into(),
            ]),
            ..Default::default()
        };

        store.update("model-o-wireless", &patch).unwrap();
        let view = store.get_current_settings_view("model-o-wireless");
        assert_eq!(view.debounce_ms, 4);
        assert_eq!(view.lod, 2);
        assert_eq!(view.active_dpi_stage, 3);
        assert_eq!(view.dpi_stages, vec![800, 1600, 3200]);
        assert_eq!(
            view.dpi_colors,
            vec!["#112233", "#445566", "#778899"]
        );
    }

    #[test]
    fn updates_buttons_via_patch() {
        let store = ProfileStore::in_memory(ProfileStore::default_data());
        let patch = SettingsPatch {
            buttons: Some(vec![ButtonBinding {
                pin: "dpi_loop".into(),
                button_id: 6,
                action: ButtonAction::Dpi { action: 1 },
            }]),
            ..Default::default()
        };

        store.update("model-o-wireless", &patch).unwrap();
        let view = store.get_current_settings_view("model-o-wireless");
        assert!(view.buttons.is_some());
        let btns = view.buttons.unwrap();
        assert_eq!(btns.len(), 1);
        assert_eq!(btns[0].button_id, 6);
        assert_eq!(btns[0].action, ButtonAction::Dpi { action: 1 });
    }
}
