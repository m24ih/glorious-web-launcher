#[cfg(target_os = "linux")]
use std::collections::HashMap;
#[cfg(target_os = "linux")]
use std::sync::Arc;

#[cfg(target_os = "linux")]
use ksni::menu::{MenuItem, StandardItem, SubMenu};
#[cfg(target_os = "linux")]
use ksni::TrayMethods;
#[cfg(target_os = "linux")]
use ksni::{Icon, ToolTip, Tray};
#[cfg(target_os = "linux")]
use tokio::sync::Notify;

#[cfg(target_os = "linux")]
use crate::api::AppState;
#[cfg(target_os = "linux")]
use crate::types::{DeviceStatusView, DeviceView, EffectSpec, RgbPatch, SettingsPatch};
#[cfg(target_os = "linux")]
use crate::ws::WsEvent;

pub const COLOR_DISCONNECTED: [u8; 4] = [255, 110, 115, 120];
pub const COLOR_CHARGING: [u8; 4] = [255, 52, 152, 219];
pub const COLOR_BATTERY_GREEN: [u8; 4] = [255, 46, 204, 113];
pub const COLOR_BATTERY_YELLOW: [u8; 4] = [255, 241, 196, 15];
pub const COLOR_BATTERY_ORANGE: [u8; 4] = [255, 230, 126, 34];
pub const COLOR_BATTERY_RED: [u8; 4] = [255, 231, 76, 60];

pub fn battery_color(connected: bool, battery: Option<(u8, bool)>) -> [u8; 4] {
    match (connected, battery) {
        (true, Some((_, true))) => COLOR_CHARGING,
        (true, Some((percent, false))) if percent <= 25 => COLOR_BATTERY_RED,
        (true, Some((percent, false))) if percent <= 60 => COLOR_BATTERY_ORANGE,
        (true, Some((percent, false))) if percent <= 90 => COLOR_BATTERY_YELLOW,
        (true, Some(_)) => COLOR_BATTERY_GREEN,
        _ => COLOR_DISCONNECTED,
    }
}

pub fn raw_mouse_icon(color: [u8; 4]) -> Vec<u8> {
    let mut data = vec![0; 32 * 32 * 4];
    for y in 0..32_i32 {
        for x in 0..32_i32 {
            let dx = x - 15;
            let dy = y - 16;
            let outer = dx * dx * 169 + dy * dy * 100;
            let inner = dx * dx * 121 + dy * dy * 64;
            let outline = outer <= 16900 && inner >= 7744;
            let divider = (y == 14 && (6..=24).contains(&x)) || (x == 15 && (4..=14).contains(&y));
            if outline || divider {
                let index = ((y * 32 + x) * 4) as usize;
                data[index..index + 4].copy_from_slice(&color);
            }
        }
    }
    data
}

#[cfg(target_os = "linux")]
fn mouse_icon(color: [u8; 4]) -> Icon {
    Icon {
        width: 32,
        height: 32,
        data: raw_mouse_icon(color),
    }
}

#[cfg(target_os = "linux")]
fn device_icon(devices: &[DeviceView]) -> Icon {
    let battery = devices
        .iter()
        .filter(|device| device.connected && device.status == DeviceStatusView::Connected)
        .filter_map(|device| {
            device
                .battery
                .as_ref()
                .filter(|battery| battery.status == "ok")
        })
        .min_by_key(|battery| battery.percentage)
        .map(|battery| (battery.percentage, battery.charging));
    mouse_icon(battery_color(battery.is_some(), battery))
}

#[derive(Debug, PartialEq, Eq)]
pub enum Notice {
    Connected,
    Disconnected,
    Charging,
    Full,
    LowBattery(u8),
}

#[derive(Default)]
pub struct DeviceState {
    pub connected: bool,
    pub charging: Option<bool>,
    pub threshold: Option<u8>,
    pub percentage: Option<u8>,
}

impl DeviceState {
    pub fn update(&mut self, connected: bool, battery: Option<(u8, bool)>) -> Vec<Notice> {
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

#[cfg(target_os = "linux")]
struct DesktopTray {
    url: String,
    devices: Vec<DeviceView>,
    shutdown: Arc<Notify>,
    app: AppState,
    settings: crate::tray_settings::TraySettings,
}

#[cfg(target_os = "linux")]
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
    fn icon_pixmap(&self) -> Vec<Icon> {
        vec![device_icon(&self.devices)]
    }
    fn tool_tip(&self) -> ToolTip {
        ToolTip {
            title: self.title(),
            description: status_lines(&self.devices).join("\n"),
            icon_pixmap: self.icon_pixmap(),
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
                    label: "Status: No device connected".into(),
                    enabled: false,
                    ..Default::default()
                }
                .into(),
            );
            items.push(
                StandardItem {
                    label: "Battery: unavailable".into(),
                    enabled: false,
                    ..Default::default()
                }
                .into(),
            );
        }
        for device in &self.devices {
            let status = match device.status {
                DeviceStatusView::Connected => "Connected",
                DeviceStatusView::Asleep => "Asleep or out of range",
                DeviceStatusView::Unsupported => "Unsupported",
                DeviceStatusView::Error => "Error",
                DeviceStatusView::Disconnected => "Disconnected",
            };
            items.push(
                StandardItem {
                    label: format!("{}: {status}", device.name),
                    enabled: false,
                    ..Default::default()
                }
                .into(),
            );
            let battery = match &device.battery {
                Some(battery) => format!(
                    "Battery: {}%{}",
                    battery.percentage,
                    if battery.charging { " (charging)" } else { "" }
                ),
                None => "Battery: unavailable".into(),
            };
            items.push(
                StandardItem {
                    label: battery,
                    enabled: false,
                    ..Default::default()
                }
                .into(),
            );
        }
        items.push(MenuItem::Separator);
        let mut device_modes = Vec::new();
        let mut mode_enabled = false;
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
                            label: "DPI".into(),
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
            let enabled = ready && !actions.is_empty();
            mode_enabled |= enabled;
            if self.devices.len() == 1 {
                device_modes = actions;
            } else {
                device_modes.push(
                    SubMenu {
                        label: device.name.clone(),
                        enabled,
                        submenu: actions,
                        ..Default::default()
                    }
                    .into(),
                );
            }
        }
        items.push(
            SubMenu {
                label: "Mode".into(),
                enabled: mode_enabled,
                submenu: device_modes,
                ..Default::default()
            }
            .into(),
        );
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

#[cfg(target_os = "linux")]
#[derive(Clone, Copy)]
enum Action {
    Dpi(u8),
    Profile(u8),
    Polling(u32),
    ToggleLighting,
    Effect(u8),
    Brightness(u8),
}

#[cfg(target_os = "linux")]
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

pub fn notify_error(name: &str, error: &str) {
    tracing::warn!(device = name, %error, "tray action failed");
    let name = name.to_string();
    let error = error.to_string();
    tokio::spawn(async move {
        let mut notif = notify_rust::Notification::new();
        notif.appname("Glorious Web Launcher")
            .summary(&name)
            .body(&error)
            .urgency(notify_rust::Urgency::Normal);
        #[cfg(target_os = "windows")]
        {
            notif.app_id("Glorious Web Launcher");
        }
        let _ = notif.show();
    });
}

#[cfg(target_os = "linux")]
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

pub fn open_browser(url: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", url])
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(url)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg(url)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }
}

pub fn notify(name: &str, notice: Notice) {
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
        let urg = match urgency {
            "low" => notify_rust::Urgency::Low,
            "critical" => notify_rust::Urgency::Critical,
            _ => notify_rust::Urgency::Normal,
        };
        let mut notif = notify_rust::Notification::new();
        notif.appname("Glorious Web Launcher")
            .summary(&name)
            .body(&body)
            .urgency(urg);
        #[cfg(target_os = "windows")]
        {
            notif.app_id("Glorious Web Launcher");
        }
        if let Err(error) = notif.show() {
            tracing::warn!(%error, "could not send desktop notification");
        }
    });
}

#[cfg(target_os = "linux")]
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

    #[cfg(target_os = "linux")]
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
        tray.devices[0].battery = Some(crate::types::BatteryInfo {
            percentage: 50,
            charging: false,
            status: "ok".into(),
        });
        assert!(device_icon(&tray.devices)
            .data
            .chunks_exact(4)
            .any(|pixel| pixel == COLOR_BATTERY_ORANGE));
        let mut low = tray.devices[0].clone();
        low.battery.as_mut().unwrap().percentage = 8;
        tray.devices.push(low);
        assert!(device_icon(&tray.devices)
            .data
            .chunks_exact(4)
            .any(|pixel| pixel == COLOR_BATTERY_RED));
        tray.devices[1].status = DeviceStatusView::Asleep;
        assert!(device_icon(&tray.devices)
            .data
            .chunks_exact(4)
            .any(|pixel| pixel == COLOR_BATTERY_ORANGE));
        tray.devices.pop();
        tray.settings.show_polling = false;
        let menu = tray.menu();
        let MenuItem::Standard(status) = &menu[2] else {
            panic!("missing status")
        };
        assert!(!status.enabled);
        let MenuItem::Standard(battery) = &menu[3] else {
            panic!("missing battery")
        };
        assert!(!battery.enabled);
        let MenuItem::SubMenu(device_menu) = &menu[5] else {
            panic!("missing mode menu")
        };
        assert_eq!(device_menu.label, "Mode");
        let labels: Vec<_> = device_menu
            .submenu
            .iter()
            .filter_map(|item| match item {
                MenuItem::SubMenu(menu) => Some(menu.label.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(labels, vec!["DPI", "Profile", "Lighting"]);
        tray.devices[0].supported = false;
        let menu = tray.menu();
        let MenuItem::SubMenu(device_menu) = &menu[5] else {
            panic!("missing device menu")
        };
        assert!(!device_menu.enabled);

        let mut second = tray.devices[0].clone();
        second.name = "Second mouse".into();
        second.slug = "second".into();
        second.supported = true;
        tray.devices.push(second);
        let menu = tray.menu();
        let MenuItem::SubMenu(mode) = &menu[7] else {
            panic!("missing multi-device mode")
        };
        assert_eq!(mode.submenu.len(), 2);
        let MenuItem::SubMenu(first) = &mode.submenu[0] else {
            panic!("missing first device")
        };
        let MenuItem::SubMenu(second) = &mode.submenu[1] else {
            panic!("missing second device")
        };
        assert!(!first.enabled && second.enabled);
    }

    #[test]
    fn icon_colors_follow_battery_and_connection() {
        assert_eq!(battery_color(false, None), COLOR_DISCONNECTED);
        assert_eq!(battery_color(true, None), COLOR_DISCONNECTED);
        assert_eq!(battery_color(true, Some((5, true))), COLOR_CHARGING);
        assert_eq!(battery_color(true, Some((0, false))), COLOR_BATTERY_RED);
        assert_eq!(battery_color(true, Some((25, false))), COLOR_BATTERY_RED);
        assert_eq!(battery_color(true, Some((26, false))), COLOR_BATTERY_ORANGE);
        assert_eq!(battery_color(true, Some((60, false))), COLOR_BATTERY_ORANGE);
        assert_eq!(battery_color(true, Some((61, false))), COLOR_BATTERY_YELLOW);
        assert_eq!(battery_color(true, Some((90, false))), COLOR_BATTERY_YELLOW);
        assert_eq!(battery_color(true, Some((91, false))), COLOR_BATTERY_GREEN);
        assert_eq!(battery_color(true, Some((100, false))), COLOR_BATTERY_GREEN);
    }

    #[test]
    fn mouse_icon_has_transparent_background_and_colored_pixels() {
        let icon_data = raw_mouse_icon(COLOR_CHARGING);
        assert_eq!(icon_data.len(), 32 * 32 * 4);
        assert!(icon_data
            .chunks_exact(4)
            .any(|pixel| pixel == COLOR_CHARGING));
        assert!(icon_data.chunks_exact(4).any(|pixel| pixel == [0, 0, 0, 0]));
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
