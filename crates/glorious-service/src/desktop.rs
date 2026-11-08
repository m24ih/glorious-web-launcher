use std::collections::HashMap;
use std::sync::Arc;

use ksni::menu::{MenuItem, StandardItem, SubMenu};
use ksni::TrayMethods;
use ksni::{ToolTip, Tray};
use tokio::sync::Notify;

use crate::api::AppState;
use crate::types::{DeviceStatusView, DeviceView, EffectSpec, RgbPatch, SettingsPatch};
use crate::ws::WsEvent;

#[derive(Debug, PartialEq, Eq)]
enum Notice {
    Connected,
    Disconnected,
    Charging,
    Full,
    LowBattery(u8),
}

#[derive(Default)]
struct DeviceState {
    connected: bool,
    charging: Option<bool>,
    threshold: Option<u8>,
    percentage: Option<u8>,
}

impl DeviceState {
    fn update(&mut self, connected: bool, battery: Option<(u8, bool)>) -> Vec<Notice> {
        let mut notices = Vec::new();
        if connected != self.connected {
            notices.push(if connected {
                Notice::Connected
            } else {
                Notice::Disconnected
            });
        }
        self.connected = connected;
        if !connected {
            self.charging = None;
            self.threshold = None;
            self.percentage = None;
            return notices;
        }
        if let Some((percent, charging)) = battery {
            if charging && self.charging == Some(false) {
                notices.push(Notice::Charging);
            } else if percent == 100 && self.charging == Some(true) && self.percentage != Some(100)
            {
                notices.push(Notice::Full);
            }
            self.charging = Some(charging);
            self.percentage = Some(percent);
            if charging || percent > 25 {
                self.threshold = None;
            }
            if !charging {
                let threshold = if percent <= 5 {
                    Some(5)
                } else if percent <= 10 {
                    Some(10)
                } else if percent <= 20 {
                    Some(20)
                } else {
                    None
                };
                if let Some(level) = threshold {
                    if self.threshold.map_or(true, |previous| level < previous) {
                        self.threshold = Some(level);
                        notices.push(Notice::LowBattery(percent));
                    }
                }
            }
        }
        notices
    }
}

struct DesktopTray {
    url: String,
    devices: Vec<DeviceView>,
    shutdown: Arc<Notify>,
    app: AppState,
    settings: crate::tray_settings::TraySettings,
}

impl Tray for DesktopTray {
    fn id(&self) -> String {
        "glorious-web-launcher".into()
    }
    fn title(&self) -> String {
        "Glorious Web Launcher".into()
    }
    fn category(&self) -> ksni::Category {
        ksni::Category::Hardware
    }
    fn icon_name(&self) -> String {
        "input-mouse".into()
    }
    fn tool_tip(&self) -> ToolTip {
        ToolTip {
            title: self.title(),
            description: status_lines(&self.devices).join("\n"),
            ..Default::default()
        }
    }
    fn activate(&mut self, _: i32, _: i32) {
        open_browser(&self.url);
    }
    fn menu(&self) -> Vec<MenuItem<Self>> {
        let mut items = vec![
            StandardItem {
                label: "Open Control Panel".into(),
                activate: Box::new(|tray: &mut Self| open_browser(&tray.url)),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
        ];
        if self.devices.is_empty() {
            items.push(
                StandardItem {
                    label: "No device connected".into(),
                    enabled: false,
                    ..Default::default()
                }
                .into(),
            );
        }
        for device in &self.devices {
            let ready = device.connected
                && device.supported
                && matches!(device.status, DeviceStatusView::Connected);
            let mut actions = Vec::new();
            if let Some(settings) = &device.settings {
                if self.settings.show_dpi {
                    let entries = settings
                        .dpi_stages
                        .iter()
                        .enumerate()
                        .map(|(i, dpi)| {
                            action_item(
                                device,
                                format!(
                                    "{}{} DPI",
                                    if settings.active_dpi_stage as usize == i + 1 {
                                        "✓ "
                                    } else {
                                        ""
                                    },
                                    dpi
                                ),
                                Action::Dpi((i + 1) as u8),
                            )
                        })
                        .collect();
                    actions.push(
                        SubMenu {
                            label: "DPI (saved stages)".into(),
                            submenu: entries,
                            enabled: !settings.dpi_stages.is_empty(),
                            ..Default::default()
                        }
                        .into(),
                    );
                }
                if self.settings.show_profiles {
                    let entries = (1..=3)
                        .map(|profile| {
                            action_item(
                                device,
                                format!(
                                    "{}Profile {profile}",
                                    if settings.active_profile == profile {
                                        "✓ "
                                    } else {
                                        ""
                                    }
                                ),
                                Action::Profile(profile),
                            )
                        })
                        .collect();
                    actions.push(
                        SubMenu {
                            label: "Profile".into(),
                            submenu: entries,
                            ..Default::default()
                        }
                        .into(),
                    );
                }
                if self.settings.show_lighting {
                    if let Some(light) = &settings.lighting {
                        let mut entries: Vec<MenuItem<Self>> = Vec::new();
                        entries.push(action_item(
                            device,
                            if light.effect == 0 {
                                "Turn on".into()
                            } else {
                                "Turn off".into()
                            },
                            Action::ToggleLighting,
                        ));
                        let effects = [
                            "Off",
                            "Glorious",
                            "Seamless Breathing",
                            "Breathing",
                            "Static",
                            "Single Breathing",
                            "Tail",
                            "Rave",
                            "Wave",
                        ];
                        let effects = effects
                            .iter()
                            .enumerate()
                            .map(|(id, name)| {
                                action_item(
                                    device,
                                    format!(
                                        "{}{name}",
                                        if light.effect == id as u8 { "✓ " } else { "" }
                                    ),
                                    Action::Effect(id as u8),
                                )
                            })
                            .collect();
                        entries.push(
                            SubMenu {
                                label: "Effect".into(),
                                submenu: effects,
                                ..Default::default()
                            }
                            .into(),
                        );
                        let brightness = [0, 25, 50, 75, 100]
                            .iter()
                            .map(|&n| {
                                action_item(
                                    device,
                                    format!(
                                        "{}{n}%",
                                        if light.brightness == n { "✓ " } else { "" }
                                    ),
                                    Action::Brightness(n),
                                )
                            })
                            .collect();
                        entries.push(
                            SubMenu {
                                label: "Brightness".into(),
                                submenu: brightness,
                                ..Default::default()
                            }
                            .into(),
                        );
                        actions.push(
                            SubMenu {
                                label: "Lighting".into(),
                                submenu: entries,
                                ..Default::default()
                            }
                            .into(),
                        );
                    }
                }
                if self.settings.show_polling {
                    let entries = [125, 250, 500, 1000]
                        .iter()
                        .map(|&rate| {
                            action_item(
                                device,
                                format!(
                                    "{}{rate} Hz",
                                    if settings.polling_rate == rate {
                                        "✓ "
                                    } else {
                                        ""
                                    }
                                ),
                                Action::Polling(rate),
                            )
                        })
                        .collect();
                    actions.push(
                        SubMenu {
                            label: "Polling rate".into(),
                            submenu: entries,
                            ..Default::default()
                        }
                        .into(),
                    );
                }
            }
            let status = status_lines(std::slice::from_ref(device)).join("");
            items.push(
                SubMenu {
                    label: status,
                    enabled: ready && !actions.is_empty(),
                    submenu: actions,
                    ..Default::default()
                }
                .into(),
            );
        }
        items.push(MenuItem::Separator);
        items.push(
            StandardItem {
                label: "Quit".into(),
                activate: Box::new(|tray: &mut Self| tray.shutdown.notify_one()),
                ..Default::default()
            }
            .into(),
        );
        items
    }
}

#[derive(Clone, Copy)]
enum Action {
    Dpi(u8),
    Profile(u8),
    Polling(u32),
    ToggleLighting,
    Effect(u8),
    Brightness(u8),
}

fn action_item(device: &DeviceView, label: String, action: Action) -> MenuItem<DesktopTray> {
    let slug = device.slug.clone();
    StandardItem { label, activate: Box::new(move |tray: &mut DesktopTray| {
        let Some(device) = tray.devices.iter().find(|d| d.slug == slug).cloned() else { return; };
        let app = tray.app.clone();
        tokio::spawn(async move {
            let name = device.name.clone();
            let result = tokio::task::spawn_blocking(move || {
                let settings = device.settings.ok_or_else(|| "Device settings unavailable".to_string())?;
                let mut patch = SettingsPatch::default();
                match action {
                    Action::Dpi(stage) => patch.active_dpi_stage = Some(stage),
                    Action::Profile(profile) => patch.profile = Some(profile),
                    Action::Polling(rate) => patch.polling_rate = Some(rate),
                    Action::ToggleLighting | Action::Effect(_) | Action::Brightness(_) => {
                        let mut light = settings.lighting.ok_or_else(|| "Lighting settings unavailable".to_string())?;
                        let key = format!("{}:{}", device.slug, settings.active_profile);
                        let effect = match action {
                            Action::ToggleLighting if light.effect != 0 => 0,
                            Action::ToggleLighting => app.tray_settings.get().last_lighting.get(&key).map(|saved| saved.effect).ok_or_else(|| "No previous lighting effect is saved; select an effect first".to_string())?,
                            Action::Effect(effect) => effect,
                            _ => light.effect,
                        };
                        if effect == 0 && light.effect != 0 {
                            app.tray_settings.remember_lighting(key.clone(), light.clone()).map_err(|error| format!("Could not save previous lighting: {error}"))?;
                        } else if matches!(action, Action::ToggleLighting) && light.effect == 0 {
                            if let Some(saved) = app.tray_settings.get().last_lighting.get(&key) { light = saved.clone(); }
                        }
                        if let Action::Brightness(brightness) = action { light.brightness = brightness; }
                        patch.rgb = Some(RgbPatch { effect: EffectSpec::Id(effect), effect_name: None, brightness: light.brightness, wireless_brightness: None, rate: light.rate, color: light.color });
                    }
                }
                let result = app.manager.apply(&device.slug, &patch).map_err(|e| e.to_string())?;
                let _ = app.tx.send(WsEvent::Applied { ts: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(), slug: result.slug.clone(), applied: result.applied.clone(), device: Box::new(result.device) });
                Ok::<(), String>(())
            }).await;
            match result {
                Ok(Ok(())) => {},
                Ok(Err(error)) => notify_error(&name, &error),
                Err(error) => notify_error(&name, &error.to_string()),
            }
        });
    }), ..Default::default() }.into()
}

fn notify_error(name: &str, error: &str) {
    tracing::warn!(device = name, %error, "tray action failed");
    let name = name.to_string();
    let error = error.to_string();
    tokio::spawn(async move {
        let _ = tokio::process::Command::new("notify-send")
            .args([
                "--app-name=Glorious Web Launcher",
                "--urgency=normal",
                &name,
                &error,
            ])
            .status()
            .await;
    });
}

fn status_lines(devices: &[DeviceView]) -> Vec<String> {
    if devices.is_empty() {
        return vec!["No device connected".into()];
    }
    devices
        .iter()
        .map(|device| match &device.battery {
            Some(battery) => format!(
                "{}: {}%{}",
                device.name,
                battery.percentage,
                if battery.charging { " (charging)" } else { "" }
            ),
            None => format!("{}: {:?}", device.name, device.status),
        })
        .collect()
}

fn open_browser(url: &str) {
    let _ = std::process::Command::new("xdg-open")
        .arg(url)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}

fn notify(name: &str, notice: Notice) {
    let (urgency, body) = match notice {
        Notice::Connected => ("low", "Connected".to_string()),
        Notice::Disconnected => ("low", "Disconnected".to_string()),
        Notice::Charging => ("low", "Charging started".to_string()),
        Notice::Full => ("normal", "Battery fully charged".to_string()),
        Notice::LowBattery(percent) => (
            if percent <= 5 { "critical" } else { "normal" },
            format!("Low battery: {percent}%"),
        ),
    };
    let name = name.to_string();
    tokio::spawn(async move {
        if let Err(error) = tokio::process::Command::new("notify-send")
            .args([
                "--app-name=Glorious Web Launcher",
                "--icon=input-mouse",
                "--urgency",
                urgency,
                &name,
                &body,
            ])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .await
        {
            tracing::warn!(%error, "could not send desktop notification");
        }
    });
}

/// Start a Linux StatusNotifierItem and subscribe to the existing telemetry stream.
pub fn spawn(state: &AppState, url: String, shutdown: Arc<Notify>) {
    let mut rx = state.tx.subscribe();
    let tray = DesktopTray {
        url,
        devices: Vec::new(),
        shutdown,
        app: state.clone(),
        settings: state.tray_settings.get(),
    };
    let settings_store = state.tray_settings.clone();
    let mut settings_rx = settings_store.subscribe();
    tokio::spawn(async move {
        let handle = match tray.assume_sni_available(true).spawn().await {
            Ok(handle) => Some(handle),
            Err(error) => {
                tracing::warn!(%error, "system tray unavailable");
                None
            }
        };
        let mut previous: HashMap<String, (String, DeviceState)> = HashMap::new();
        loop {
            let event = tokio::select! {
                event = rx.recv() => event,
                changed = settings_rx.changed() => {
                    if changed.is_err() { break; }
                    let settings = settings_rx.borrow_and_update().clone();
                    if let Some(handle) = &handle { handle.update(|tray| tray.settings = settings).await; }
                    continue;
                }
            };
            match event {
                Ok(WsEvent::Telemetry { devices, .. }) => {
                    let mut seen = std::collections::HashSet::new();
                    for device in &devices {
                        seen.insert(device.slug.clone());
                        let (_, state) = previous
                            .entry(device.slug.clone())
                            .or_insert_with(|| (device.name.clone(), DeviceState::default()));
                        let battery = device
                            .battery
                            .as_ref()
                            .filter(|b| b.status == "ok")
                            .map(|b| (b.percentage, b.charging));
                        for notice in state.update(device.connected, battery) {
                            if settings_store.get().notifications {
                                notify(&device.name, notice);
                            }
                        }
                    }
                    for (slug, (name, state)) in &mut previous {
                        if !seen.contains(slug) {
                            for notice in state.update(false, None) {
                                if settings_store.get().notifications {
                                    notify(name, notice);
                                }
                            }
                        }
                    }
                    if let Some(handle) = &handle {
                        handle.update(|tray| tray.devices = devices).await;
                    }
                }
                Ok(WsEvent::Applied { device, .. }) => {
                    if let Some(handle) = &handle {
                        handle
                            .update(|tray| {
                                if let Some(existing) =
                                    tray.devices.iter_mut().find(|d| d.slug == device.slug)
                                {
                                    *existing = *device;
                                }
                            })
                            .await;
                    }
                }
                Ok(_) | Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_shows_configured_shortcuts_only_for_supported_connected_devices() {
        let (db, _) = crate::devices::DeviceDatabase::load_default().unwrap();
        let app = AppState::new(::std::sync::Arc::new(crate::manager::DeviceManager::new(
            db,
            std::time::Duration::from_millis(1),
        )));
        let device = DeviceView {
            slug: "test".into(),
            name: "Test mouse".into(),
            id: "1".into(),
            router_id: "test".into(),
            protocol: "sinowealth".into(),
            supported: true,
            connected: true,
            status: DeviceStatusView::Connected,
            connection: None,
            image: None,
            battery: None,
            firmware: None,
            settings: Some(crate::types::SettingsView {
                dpi_stages: vec![800, 1600],
                lighting: Some(crate::types::LightingView {
                    effect: 4,
                    effect_name: "static".into(),
                    brightness: 50,
                    rate: 50,
                    color: [1, 2, 3],
                }),
                ..Default::default()
            }),
            profiles: None,
            error: None,
            last_seen: None,
        };
        let mut tray = DesktopTray {
            url: "http://localhost".into(),
            devices: vec![device],
            shutdown: Arc::new(Notify::new()),
            app,
            settings: Default::default(),
        };
        tray.settings.show_polling = false;
        let menu = tray.menu();
        let MenuItem::SubMenu(device_menu) = &menu[2] else {
            panic!("missing device menu")
        };
        let labels: Vec<_> = device_menu
            .submenu
            .iter()
            .filter_map(|item| match item {
                MenuItem::SubMenu(menu) => Some(menu.label.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(labels, vec!["DPI (saved stages)", "Profile", "Lighting"]);
        tray.devices[0].supported = false;
        let menu = tray.menu();
        let MenuItem::SubMenu(device_menu) = &menu[2] else {
            panic!("missing device menu")
        };
        assert!(!device_menu.enabled);
    }

    #[test]
    fn battery_alerts_once_per_threshold_and_rearms_after_recharge() {
        let mut state = DeviceState::default();
        assert_eq!(
            state.update(true, Some((19, false))),
            vec![Notice::Connected, Notice::LowBattery(19)]
        );
        assert!(state.update(true, Some((18, false))).is_empty());
        assert_eq!(
            state.update(true, Some((9, false))),
            vec![Notice::LowBattery(9)]
        );
        assert_eq!(state.update(true, Some((9, true))), vec![Notice::Charging]);
        assert_eq!(
            state.update(true, Some((19, false))),
            vec![Notice::LowBattery(19)]
        );
    }

    #[test]
    fn disconnect_and_full_charge_are_transitions() {
        let mut state = DeviceState::default();
        state.update(true, Some((90, true)));
        assert_eq!(state.update(true, Some((100, false))), vec![Notice::Full]);
        assert_eq!(state.update(false, None), vec![Notice::Disconnected]);
        assert!(state.update(false, None).is_empty());
    }
}
