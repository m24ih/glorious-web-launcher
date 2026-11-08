use std::collections::HashMap;
use std::sync::Arc;

use ksni::menu::{MenuItem, StandardItem};
use ksni::TrayMethods;
use ksni::{ToolTip, Tray};
use tokio::sync::Notify;

use crate::api::AppState;
use crate::types::DeviceView;
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
        items.extend(status_lines(&self.devices).into_iter().map(|label| {
            StandardItem {
                label,
                enabled: false,
                ..Default::default()
            }
            .into()
        }));
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
    };
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
            match rx.recv().await {
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
                            notify(&device.name, notice);
                        }
                    }
                    for (slug, (name, state)) in &mut previous {
                        if !seen.contains(slug) {
                            for notice in state.update(false, None) {
                                notify(name, notice);
                            }
                        }
                    }
                    if let Some(handle) = &handle {
                        handle.update(|tray| tray.devices = devices).await;
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
