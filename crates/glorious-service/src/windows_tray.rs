//! Windows System Tray implementation using tray-icon and muda.

use std::sync::Arc;
use tokio::sync::Notify;
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use muda::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, PostQuitMessage, PostThreadMessageW, TranslateMessage, MSG,
    WM_USER,
};
use crossbeam_channel::unbounded;

use crate::api::AppState;
use crate::desktop::{battery_color, notify, notify_error, open_browser, raw_mouse_icon, DeviceState, Notice};
use crate::tray_settings::TraySettings;
use crate::types::{DeviceStatusView, DeviceView, EffectSpec, RgbPatch, SettingsPatch};
use crate::ws::WsEvent;

const WM_TRAY_MSG: u32 = WM_USER + 101;

enum TrayCommand {
    UpdateDevices(Vec<DeviceView>),
    UpdateSettings(TraySettings),
    Quit,
}

pub fn spawn(state: &AppState, url: String, shutdown: Arc<Notify>) {
    let (cmd_tx, cmd_rx) = unbounded::<TrayCommand>();
    let (tid_tx, tid_rx) = std::sync::mpsc::channel::<u32>();

    let app = state.clone();
    let tray_url = url.clone();
    let tray_shutdown = shutdown.clone();

    // Spawn dedicated OS thread for Win32 message pump
    std::thread::spawn(move || {
        let thread_id = unsafe { GetCurrentThreadId() };
        let _ = tid_tx.send(thread_id);

        let mut settings = app.tray_settings.get();
        let mut devices: Vec<DeviceView> = Vec::new();

        let initial_icon = mouse_tray_icon(battery_color(false, None)).unwrap_or_else(|| {
            Icon::from_rgba(vec![0; 32 * 32 * 4], 32, 32).unwrap()
        });

        let menu = build_menu(&devices, &settings);

        let tray_icon = match TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Glorious Web Launcher")
            .with_icon(initial_icon)
            .build()
        {
            Ok(tray) => Some(tray),
            Err(err) => {
                tracing::warn!(error = %err, "could not create Windows tray icon");
                None
            }
        };

        let mut msg: MSG = unsafe { std::mem::zeroed() };
        loop {
            let res = unsafe { GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) };
            if res <= 0 {
                break;
            }

            if msg.message == WM_TRAY_MSG {
                while let Ok(cmd) = cmd_rx.try_recv() {
                    match cmd {
                        TrayCommand::UpdateDevices(new_devices) => {
                            devices = new_devices;
                            if let Some(tray) = &tray_icon {
                                let color = device_battery_color(&devices);
                                if let Some(icon) = mouse_tray_icon(color) {
                                    let _ = tray.set_icon(Some(icon));
                                }
                                let tooltip = status_tooltip(&devices);
                                let _ = tray.set_tooltip(Some(tooltip));
                                let new_menu = build_menu(&devices, &settings);
                                tray.set_menu(Some(Box::new(new_menu)));
                            }
                        }
                        TrayCommand::UpdateSettings(new_settings) => {
                            settings = new_settings;
                            if let Some(tray) = &tray_icon {
                                let new_menu = build_menu(&devices, &settings);
                                tray.set_menu(Some(Box::new(new_menu)));
                            }
                        }
                        TrayCommand::Quit => {
                            unsafe { PostQuitMessage(0) };
                        }
                    }
                }
            }

            unsafe {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            // Handle Menu events
            while let Ok(event) = MenuEvent::receiver().try_recv() {
                handle_menu_action(event.id().as_ref(), &devices, &app, &tray_url, &tray_shutdown);
            }

            // Handle Tray click events (Left click -> Open Web UI)
            while let Ok(event) = TrayIconEvent::receiver().try_recv() {
                if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                    open_browser(&tray_url);
                }
            }
        }
    });

    let thread_id = match tid_rx.recv() {
        Ok(id) => id,
        Err(_) => return,
    };

    // Forward telemetry & preferences from tokio to the tray thread
    let tx_telemetry = cmd_tx.clone();
    let mut rx = state.tx.subscribe();
    let settings_store = state.tray_settings.clone();
    let mut settings_rx = settings_store.subscribe();
    let shutdown_notify = shutdown.clone();

    tokio::spawn(async move {
        let mut previous: std::collections::HashMap<String, (String, DeviceState)> = std::collections::HashMap::new();

        loop {
            tokio::select! {
                event = rx.recv() => {
                    let Ok(event) = event else { break; };
                    if let WsEvent::Telemetry { devices, .. } = event {
                        // Check notifications
                        let mut active_slugs = std::collections::HashSet::new();
                        let show_notifs = settings_store.get().notifications;

                        for device in &devices {
                            active_slugs.insert(device.slug.clone());
                            let battery = device.battery.as_ref().filter(|b| b.status == "ok").map(|b| (b.percentage, b.charging));
                            let is_connected = device.connected && device.status == DeviceStatusView::Connected;
                            let entry = previous.entry(device.slug.clone()).or_insert_with(|| (device.name.clone(), DeviceState::default()));
                            let notices = entry.1.update(is_connected, battery);
                            if show_notifs {
                                for notice in notices {
                                    notify(&device.name, notice);
                                }
                            }
                        }

                        // Remove disconnected devices
                        previous.retain(|slug, (name, state)| {
                            if active_slugs.contains(slug) {
                                true
                            } else {
                                if show_notifs && state.connected {
                                    notify(name, Notice::Disconnected);
                                }
                                false
                            }
                        });

                        let _ = tx_telemetry.send(TrayCommand::UpdateDevices(devices));
                        unsafe { PostThreadMessageW(thread_id, WM_TRAY_MSG, 0, 0) };
                    }
                }
                changed = settings_rx.changed() => {
                    if changed.is_err() { break; }
                    let settings = settings_rx.borrow_and_update().clone();
                    let _ = tx_telemetry.send(TrayCommand::UpdateSettings(settings));
                    unsafe { PostThreadMessageW(thread_id, WM_TRAY_MSG, 0, 0) };
                }
                _ = shutdown_notify.notified() => {
                    let _ = tx_telemetry.send(TrayCommand::Quit);
                    unsafe { PostThreadMessageW(thread_id, WM_TRAY_MSG, 0, 0) };
                    break;
                }
            }
        }
    });
}

fn mouse_tray_icon(color: [u8; 4]) -> Option<Icon> {
    let argb = raw_mouse_icon(color);
    let mut rgba = vec![0u8; 32 * 32 * 4];
    for i in 0..(32 * 32) {
        let a = argb[i * 4];
        let r = argb[i * 4 + 1];
        let g = argb[i * 4 + 2];
        let b = argb[i * 4 + 3];
        rgba[i * 4] = r;
        rgba[i * 4 + 1] = g;
        rgba[i * 4 + 2] = b;
        rgba[i * 4 + 3] = a;
    }
    Icon::from_rgba(rgba, 32, 32).ok()
}

fn device_battery_color(devices: &[DeviceView]) -> [u8; 4] {
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
    battery_color(battery.is_some(), battery)
}

fn status_tooltip(devices: &[DeviceView]) -> String {
    if devices.is_empty() {
        return "Glorious Web Launcher: No device connected".to_string();
    }
    let parts: Vec<String> = devices
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
        .collect();
    format!("Glorious Web Launcher\n{}", parts.join("\n"))
}

fn build_menu(devices: &[DeviceView], settings: &TraySettings) -> Menu {
    let menu = Menu::new();

    let _ = menu.append(&MenuItem::with_id("open_web", "Open Web UI", true, None));

    // Preferences
    let pref_submenu = Submenu::new("Preferences", true);
    let _ = pref_submenu.append(&CheckMenuItem::with_id("pref:dpi", "Show DPI Stages", true, settings.show_dpi, None));
    let _ = pref_submenu.append(&CheckMenuItem::with_id("pref:profiles", "Show Profiles", true, settings.show_profiles, None));
    let _ = pref_submenu.append(&CheckMenuItem::with_id("pref:lighting", "Show Lighting", true, settings.show_lighting, None));
    let _ = pref_submenu.append(&CheckMenuItem::with_id("pref:polling", "Show Polling Rate", true, settings.show_polling, None));
    let _ = pref_submenu.append(&CheckMenuItem::with_id("pref:notifications", "Desktop Notifications", true, settings.notifications, None));
    let _ = menu.append(&pref_submenu);

    let _ = menu.append(&PredefinedMenuItem::separator());

    // Devices overview
    if devices.is_empty() {
        let _ = menu.append(&MenuItem::new("No device connected", false, None));
    }
    for device in devices {
        let status = match device.status {
            DeviceStatusView::Connected => "Connected",
            DeviceStatusView::Asleep => "Asleep or out of range",
            DeviceStatusView::Unsupported => "Unsupported",
            DeviceStatusView::Error => "Error",
            DeviceStatusView::Disconnected => "Disconnected",
        };
        let _ = menu.append(&MenuItem::new(format!("{}: {status}", device.name), false, None));
        let battery_str = match &device.battery {
            Some(battery) => format!(
                "Battery: {}%{}",
                battery.percentage,
                if battery.charging { " (charging)" } else { "" }
            ),
            None => "Battery: unavailable".into(),
        };
        let _ = menu.append(&MenuItem::new(battery_str, false, None));
    }

    let _ = menu.append(&PredefinedMenuItem::separator());

    // Controls
    for device in devices {
        if !device.connected || !device.supported || !matches!(device.status, DeviceStatusView::Connected) {
            continue;
        }
        let Some(dev_settings) = &device.settings else { continue; };

        if settings.show_dpi {
            let dpi_sub = Submenu::new(format!("{} DPI", device.name), true);
            for (i, dpi) in dev_settings.dpi_stages.iter().enumerate() {
                let stage = (i + 1) as u8;
                let is_active = dev_settings.active_dpi_stage == stage;
                let id = format!("action:{}:dpi:{}", device.slug, stage);
                let label = format!("{dpi} DPI");
                let _ = dpi_sub.append(&CheckMenuItem::with_id(id, label, true, is_active, None));
            }
            let _ = menu.append(&dpi_sub);
        }

        if settings.show_profiles {
            let prof_sub = Submenu::new(format!("{} Profile", device.name), true);
            for p in 1..=3 {
                let is_active = dev_settings.active_profile == p;
                let id = format!("action:{}:profile:{}", device.slug, p);
                let label = format!("Profile {p}");
                let _ = prof_sub.append(&CheckMenuItem::with_id(id, label, true, is_active, None));
            }
            let _ = menu.append(&prof_sub);
        }

        if settings.show_polling {
            let poll_sub = Submenu::new(format!("{} Polling Rate", device.name), true);
            for &rate in &[125, 250, 500, 1000] {
                let is_active = dev_settings.polling_rate == rate;
                let id = format!("action:{}:polling:{}", device.slug, rate);
                let label = format!("{rate} Hz");
                let _ = poll_sub.append(&CheckMenuItem::with_id(id, label, true, is_active, None));
            }
            let _ = menu.append(&poll_sub);
        }

        if settings.show_lighting {
            if let Some(light) = &dev_settings.lighting {
                let light_sub = Submenu::new(format!("{} Lighting", device.name), true);
                let toggle_label = if light.effect != 0 { "Turn Off Lighting" } else { "Turn On Lighting" };
                let _ = light_sub.append(&MenuItem::with_id(format!("action:{}:toggle_light", device.slug), toggle_label, true, None));

                let effects_sub = Submenu::new("Effects", true);
                for &(effect_id, effect_label) in &[
                    (1, "Glorious Mode"),
                    (2, "Seamless Breathing"),
                    (3, "Breathing"),
                    (4, "Single Color"),
                    (5, "Breathing (Single Color)"),
                    (6, "Tail"),
                    (7, "Rave"),
                    (8, "Wave"),
                ] {
                    let is_active = light.effect == effect_id;
                    let id = format!("action:{}:effect:{}", device.slug, effect_id);
                    let _ = effects_sub.append(&CheckMenuItem::with_id(id, effect_label, true, is_active, None));
                }
                let _ = light_sub.append(&effects_sub);

                let bright_sub = Submenu::new("Brightness", true);
                for &(pct, label) in &[(0, "0%"), (25, "25%"), (50, "50%"), (75, "75%"), (100, "100%")] {
                    let is_active = light.brightness == pct;
                    let id = format!("action:{}:brightness:{}", device.slug, pct);
                    let _ = bright_sub.append(&CheckMenuItem::with_id(id, label, true, is_active, None));
                }
                let _ = light_sub.append(&bright_sub);

                let _ = menu.append(&light_sub);
            }
        }
    }

    let _ = menu.append(&PredefinedMenuItem::separator());
    let _ = menu.append(&MenuItem::with_id("quit", "Quit", true, None));

    menu
}

fn handle_menu_action(
    id: &str,
    devices: &[DeviceView],
    app: &AppState,
    url: &str,
    shutdown: &Arc<Notify>,
) {
    if id == "open_web" {
        open_browser(url);
        return;
    }
    if id == "quit" {
        shutdown.notify_one();
        return;
    }
    if let Some(pref) = id.strip_prefix("pref:") {
        let store = &app.tray_settings;
        let mut curr = store.get();
        match pref {
            "dpi" => curr.show_dpi = !curr.show_dpi,
            "profiles" => curr.show_profiles = !curr.show_profiles,
            "lighting" => curr.show_lighting = !curr.show_lighting,
            "polling" => curr.show_polling = !curr.show_polling,
            "notifications" => curr.notifications = !curr.notifications,
            _ => return,
        }
        let _ = store.set(curr);
        return;
    }
    if let Some(action_str) = id.strip_prefix("action:") {
        let parts: Vec<&str> = action_str.split(':').collect();
        if parts.is_empty() {
            return;
        }
        let slug = parts[0].to_string();
        let action_type = parts.get(1).copied().unwrap_or("").to_string();
        let val_str = parts.get(2).copied().unwrap_or("").to_string();

        let Some(device) = devices.iter().find(|d| d.slug == slug).cloned() else {
            return;
        };
        let app = app.clone();

        tokio::spawn(async move {
            let name = device.name.clone();
            let result = tokio::task::spawn_blocking(move || {
                let settings = device.settings.ok_or_else(|| "Device settings unavailable".to_string())?;
                let mut patch = SettingsPatch::default();
                match action_type.as_str() {
                    "dpi" => {
                        let stage: u8 = val_str.parse().map_err(|e: std::num::ParseIntError| e.to_string())?;
                        patch.active_dpi_stage = Some(stage);
                    }
                    "profile" => {
                        let profile: u8 = val_str.parse().map_err(|e: std::num::ParseIntError| e.to_string())?;
                        patch.profile = Some(profile);
                    }
                    "polling" => {
                        let rate: u32 = val_str.parse().map_err(|e: std::num::ParseIntError| e.to_string())?;
                        patch.polling_rate = Some(rate);
                    }
                    "toggle_light" | "effect" | "brightness" => {
                        let mut light = settings.lighting.ok_or_else(|| "Lighting settings unavailable".to_string())?;
                        let key = format!("{}:{}", device.slug, settings.active_profile);
                        let effect = match action_type.as_str() {
                            "toggle_light" if light.effect != 0 => 0,
                            "toggle_light" => app.tray_settings.get().last_lighting.get(&key).map(|saved| saved.effect).ok_or_else(|| "No previous lighting effect is saved; select an effect first".to_string())?,
                            "effect" => val_str.parse().unwrap_or(light.effect),
                            _ => light.effect,
                        };
                        if effect == 0 && light.effect != 0 {
                            let _ = app.tray_settings.remember_lighting(key.clone(), light.clone());
                        } else if action_type == "toggle_light" && effect != 0 {
                            if let Some(saved) = app.tray_settings.get().last_lighting.get(&key) {
                                light = saved.clone();
                            }
                        }
                        if action_type == "brightness" {
                            let b: u8 = val_str.parse().unwrap_or(light.brightness);
                            light.brightness = b;
                        }
                        patch.rgb = Some(RgbPatch {
                            effect: EffectSpec::Id(effect),
                            effect_name: None,
                            brightness: light.brightness,
                            wireless_brightness: None,
                            rate: light.rate,
                            color: light.color,
                        });
                    }
                    _ => return Ok(()),
                }

                let result = app.manager.apply(&device.slug, &patch).map_err(|e| e.to_string())?;
                let _ = app.tx.send(WsEvent::Applied {
                    ts: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
                    slug: result.slug.clone(),
                    applied: result.applied.clone(),
                    device: Box::new(result.device),
                });
                Ok::<(), String>(())
            }).await;

            match result {
                Ok(Ok(())) => {},
                Ok(Err(error)) => notify_error(&name, &error),
                Err(error) => notify_error(&name, &error.to_string()),
            }
        });
    }
}
