use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tokio::sync::watch;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct TraySettings {
    pub show_dpi: bool,
    pub show_profiles: bool,
    pub show_lighting: bool,
    pub show_polling: bool,
    pub notifications: bool,
    #[serde(default)]
    pub last_lighting: std::collections::BTreeMap<String, crate::types::LightingView>,
}
impl Default for TraySettings {
    fn default() -> Self {
        Self {
            show_dpi: true,
            show_profiles: true,
            show_lighting: true,
            show_polling: true,
            notifications: true,
            last_lighting: Default::default(),
        }
    }
}

pub struct TraySettingsStore {
    path: PathBuf,
    value: Mutex<TraySettings>,
    writable: bool,
    updates: watch::Sender<TraySettings>,
}
impl TraySettingsStore {
    pub fn load_default() -> Self {
        Self::load(crate::profiles::ProfileStore::default_path().with_file_name("tray.json"))
    }
    pub fn load(path: PathBuf) -> Self {
        match Self::try_load(path.clone()) {
            Ok(store) => store,
            Err(error) => {
                tracing::error!(%error, path = %path.display(), "cannot read tray settings; changes disabled");
                Self::new(path, TraySettings::default(), false)
            }
        }
    }
    fn new(path: PathBuf, value: TraySettings, writable: bool) -> Self {
        let (updates, _) = watch::channel(value.clone());
        Self {
            path,
            value: Mutex::new(value),
            writable,
            updates,
        }
    }
    pub fn try_load(path: PathBuf) -> std::io::Result<Self> {
        let value = match std::fs::read(&path) {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(std::io::Error::other)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => TraySettings::default(),
            Err(error) => return Err(error),
        };
        Ok(Self::new(path, value, true))
    }
    pub fn get(&self) -> TraySettings {
        self.value.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
    pub fn subscribe(&self) -> watch::Receiver<TraySettings> {
        self.updates.subscribe()
    }
    pub fn set(&self, next: TraySettings) -> std::io::Result<()> {
        self.update(|current| {
            let last_lighting = std::mem::take(&mut current.last_lighting);
            *current = next;
            current.last_lighting = last_lighting;
        })
    }
    pub fn remember_lighting(
        &self,
        key: String,
        lighting: crate::types::LightingView,
    ) -> std::io::Result<()> {
        self.update(|current| {
            current.last_lighting.insert(key, lighting);
        })
    }
    fn update(&self, change: impl FnOnce(&mut TraySettings)) -> std::io::Result<()> {
        if !self.writable {
            return Err(std::io::Error::other(
                "tray settings file is unreadable; fix it before saving",
            ));
        }
        let mut value = self.value.lock().unwrap_or_else(|e| e.into_inner());
        let mut next = value.clone();
        change(&mut next);
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let temp = self
            .path
            .with_extension(format!("{}.tmp", std::process::id()));
        let result = (|| {
            std::fs::write(&temp, serde_json::to_vec_pretty(&next)?)?;
            std::fs::rename(&temp, &self.path)
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(&temp);
        }
        result?;
        *value = next.clone();
        self.updates.send_replace(next);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saving_preferences_preserves_saved_lighting_and_notifies_subscribers() {
        let dir =
            std::env::temp_dir().join(format!("glorious-tray-light-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("tray.json");
        let store = TraySettingsStore::load(path.clone());
        let mut updates = store.subscribe();
        let old_settings = store.get();
        let light = crate::types::LightingView {
            effect: 4,
            effect_name: "static".into(),
            brightness: 50,
            rate: 20,
            color: [1, 2, 3],
        };
        store
            .remember_lighting("mouse:1".into(), light.clone())
            .unwrap();
        store.set(old_settings).unwrap();
        assert!(updates.has_changed().unwrap());
        assert_eq!(
            updates.borrow_and_update().last_lighting.get("mouse:1"),
            Some(&light)
        );
        assert_eq!(
            TraySettingsStore::load(path)
                .get()
                .last_lighting
                .get("mouse:1"),
            Some(&light)
        );
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn settings_round_trip_and_invalid_json_does_not_overwrite_file() {
        let dir = std::env::temp_dir().join(format!("glorious-tray-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("tray.json");
        let store = TraySettingsStore::load(path.clone());
        let mut settings = store.get();
        settings.show_dpi = false;
        store.set(settings.clone()).unwrap();
        assert_eq!(TraySettingsStore::load(path.clone()).get(), settings);
        std::fs::write(&path, "invalid").unwrap();
        assert!(TraySettingsStore::try_load(path.clone()).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "invalid");
        std::fs::remove_dir_all(dir).unwrap();
    }
}
