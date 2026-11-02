//! Sinowealth vendor protocol used by the Model O / Model D families.
//!
//! # Report layout (65 bytes, report id `0x00`)
//!
//! Request (`host -> device`):
//!
//! | byte | meaning |
//! |------|---------|
//! | 0 | report id (`0x00`) |
//! | 1-2 | reserved |
//! | 3 | target: `0x00` dongle, `0x02` mouse |
//! | 4 | payload length |
//! | 5 | category: `0x00` system, `0x01` sensor, `0x02` RGB, `0x03` buttons, `0x04` macro |
//! | 6 | opcode |
//! | 7+ | payload |
//!
//! Response (`device -> host`), after dropping the report-id byte:
//!
//! | index | meaning |
//! |-------|---------|
//! | 0 | status: `0xA1` ok, `0xA0` busy, `0xA2` failure, `0xA3` unsupported, `0xA4` no RF link |
//! | 5 | opcode echo |
//! | 6+ | payload |
//!
//! The dongle needs ~50-100 ms between `HIDIOCSFEATURE` and `HIDIOCGFEATURE` to
//! relay a request to the mouse; see [`crate::hid::DEFAULT_DELAY`].

use std::thread;
use std::time::Duration;

use hidapi::{HidApi, HidDevice};

use crate::devices::{Connection, DeviceEntry};
use crate::hid::{self, ConnectionKind, DeviceStatus, Endpoint, HidError, REPORT_ID, REPORT_LEN};
use crate::types::{BatteryInfo, FirmwareInfo, RgbPatch, SettingsPatch, SettingsView};

/// Request target: the 2.4 GHz receiver itself.
pub const TARGET_DONGLE: u8 = 0x00;
/// Request target: the mouse (through the receiver when wireless).
pub const TARGET_MOUSE: u8 = 0x02;

pub const CATEGORY_SYSTEM: u8 = 0x00;
pub const CATEGORY_SENSOR: u8 = 0x01;
pub const CATEGORY_RGB: u8 = 0x02;
pub const CATEGORY_BUTTONS: u8 = 0x03;
pub const CATEGORY_MACRO: u8 = 0x04;

/// Opcodes used by the Model O / D generation.
pub mod opcodes {
    // System / status (category 0x00)
    pub const PROFILE_SET: u8 = 0x05;
    pub const SLEEP_SET: u8 = 0x07;
    pub const DEBOUNCE_SET: u8 = 0x08;
    pub const FIRMWARE: u8 = 0x81;
    pub const BATTERY: u8 = 0x83;
    pub const PROFILE_GET: u8 = 0x85;
    pub const SLEEP_GET: u8 = 0x87;
    pub const DEBOUNCE_GET: u8 = 0x88;

    // Sensor / performance (category 0x01)
    pub const POLLING_SET: u8 = 0x00;
    pub const DPI_VALUES_SET: u8 = 0x01;
    pub const ACTIVE_DPI_SET: u8 = 0x02;
    pub const LOD_SET: u8 = 0x08;
    pub const POLLING_GET: u8 = 0x80;
    pub const DPI_VALUES_GET: u8 = 0x81;
    pub const ACTIVE_DPI_GET: u8 = 0x82;
    pub const LOD_GET: u8 = 0x88;
    /// Newer firmware (Model O 2 Pro generation) uses separate per-profile
    /// polling values.
    pub const POLLING_SET_V2: u8 = 0x0A;

    // RGB (category 0x02)
    pub const RGB_EFFECT_SET: u8 = 0x00;
    pub const RGB_DPI_COLOR_SET: u8 = 0x01;
    pub const RGB_BRIGHTNESS_SET: u8 = 0x02;
}

/// Build a 65-byte host request.
pub fn build_request(target: u8, category: u8, opcode: u8, payload: &[u8]) -> [u8; REPORT_LEN] {
    let mut buf = [0u8; REPORT_LEN];
    buf[0] = REPORT_ID;
    buf[3] = target;
    buf[4] = payload.len().min(u8::MAX as usize) as u8;
    buf[5] = category;
    buf[6] = opcode;
    let take = payload.len().min(REPORT_LEN - 7);
    buf[7..7 + take].copy_from_slice(&payload[..take]);
    buf
}

/// A decoded response. `data` always starts at the status byte, regardless of
/// whether hidapi returned the report id or not.
#[derive(Debug, Clone)]
pub struct Response {
    data: Vec<u8>,
}

impl Response {
    /// Normalise a raw feature report.
    ///
    /// hidapi on Linux returns the report id in byte 0 and the report data from
    /// byte 1. Some backends strip the id; detect that by looking for the opcode
    /// echo at index 5 (no id) or 6 (with id).
    pub fn normalize(raw: &[u8], expected_opcode: u8) -> Self {
        let has_report_id = raw.len() == REPORT_LEN && raw.first() == Some(&REPORT_ID);
        let echo_with_id = raw.get(6) == Some(&expected_opcode);
        let echo_without_id = raw.get(5) == Some(&expected_opcode);

        let data = if echo_with_id || (has_report_id && !echo_without_id) {
            raw[1..].to_vec()
        } else {
            raw.to_vec()
        };
        Self { data }
    }

    pub fn status_byte(&self) -> u8 {
        self.data.first().copied().unwrap_or(0)
    }

    pub fn status(&self) -> DeviceStatus {
        DeviceStatus::from_byte(self.status_byte())
    }

    pub fn echo(&self) -> Option<u8> {
        self.data.get(5).copied()
    }

    pub fn byte(&self, index: usize) -> Option<u8> {
        self.data.get(index).copied()
    }

    pub fn payload(&self) -> &[u8] {
        self.data.get(6..).unwrap_or(&[])
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }
}

fn ensure_success(response: &Response, opcode: u8) -> Result<(), HidError> {
    if response.echo() != Some(opcode) {
        return Err(HidError::UnexpectedResponse(format!(
            "opcode echo mismatch: expected 0x{opcode:02X}, got {:?}",
            response.echo()
        )));
    }
    if response.status().is_success() {
        Ok(())
    } else {
        Err(HidError::Protocol {
            status: response.status_byte(),
            detail: response.status().as_str().to_string(),
        })
    }
}

// ---------------------------------------------------------------------------
// Parsers
// ---------------------------------------------------------------------------

/// Battery payload: `[charging, percentage]`.
pub fn parse_battery(response: &Response) -> Result<BatteryInfo, HidError> {
    ensure_success(response, opcodes::BATTERY)?;
    let charging = response.byte(6).unwrap_or(0) != 0;
    let mut percentage = response.byte(7).unwrap_or(0);
    if percentage == 0 {
        // Glorious CORE maps a reported 0 to 1 (firmware bug on some builds).
        percentage = 1;
    }
    if percentage > 100 {
        return Err(HidError::UnexpectedResponse(format!(
            "battery percentage {percentage} outside 0-100"
        )));
    }
    Ok(BatteryInfo {
        percentage,
        charging,
        status: response.status().as_str().to_string(),
    })
}

/// Firmware payload: four bytes at index 6-9 (`0.3.8.1`).
pub fn parse_firmware(response: &Response) -> Result<String, HidError> {
    ensure_success(response, opcodes::FIRMWARE)?;
    let bytes = response
        .as_slice()
        .get(6..10)
        .ok_or_else(|| HidError::UnexpectedResponse("firmware response too short".into()))?;
    if bytes.iter().all(|b| *b == 0) {
        return Err(HidError::UnexpectedResponse(
            "firmware response is all zeroes".into(),
        ));
    }
    Ok(format!(
        "{}.{}.{}.{}",
        bytes[0], bytes[1], bytes[2], bytes[3]
    ))
}

/// Active profile id (1-3) at index 6.
pub fn parse_profile(response: &Response) -> Result<u8, HidError> {
    ensure_success(response, opcodes::PROFILE_GET)?;
    response
        .byte(6)
        .filter(|p| (1..=5).contains(p))
        .ok_or_else(|| HidError::UnexpectedResponse("missing active profile".into()))
}

/// Sleep timer in seconds, `None` when disabled (`0xFFFF` or `0`).
pub fn parse_sleep_seconds(response: &Response) -> Result<Option<u32>, HidError> {
    ensure_success(response, opcodes::SLEEP_GET)?;
    let hi = response.byte(6).unwrap_or(0) as u32;
    let lo = response.byte(7).unwrap_or(0) as u32;
    let seconds = (hi << 8) | lo;
    if seconds == 0 || seconds == 0xFFFF {
        Ok(None)
    } else {
        Ok(Some(seconds))
    }
}

/// DPI stage count and resolution values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DpiInfo {
    pub stage_count: u8,
    pub stages: Vec<u32>,
}

/// DPI stage values. Count is at index 7, then `[x_hi, x_lo, y_hi, y_lo]`.
///
/// Some Model O Wireless firmware builds answer with a valid stage count but
/// zeroed resolutions; `stages` is then empty while `stage_count` is preserved.
pub fn parse_dpi_info(response: &Response) -> Result<DpiInfo, HidError> {
    ensure_success(response, opcodes::DPI_VALUES_GET)?;
    let count = response.byte(7).unwrap_or(0);
    if count == 0 || count > 8 {
        return Err(HidError::UnexpectedResponse(format!(
            "invalid DPI stage count {count}"
        )));
    }

    let mut stages = Vec::new();
    for index in 0..count as usize {
        let hi = response.byte(8 + index * 4).unwrap_or(0) as u32;
        let lo = response.byte(9 + index * 4).unwrap_or(0) as u32;
        let dpi = (hi << 8) | lo;
        if (100..=30000).contains(&dpi) {
            stages.push(dpi);
        }
    }

    Ok(DpiInfo {
        stage_count: count,
        stages,
    })
}

/// Convenience wrapper returning only the resolution values.
pub fn parse_dpi_stages(response: &Response) -> Result<Vec<u32>, HidError> {
    Ok(parse_dpi_info(response)?.stages)
}

/// 1-based active DPI stage at index 7.
pub fn parse_active_dpi_stage(response: &Response) -> Result<u8, HidError> {
    ensure_success(response, opcodes::ACTIVE_DPI_GET)?;
    response
        .byte(7)
        .filter(|stage| (1..=8).contains(stage))
        .ok_or_else(|| HidError::UnexpectedResponse("missing active DPI stage".into()))
}

/// Polling rate readback is the HID selector at index 6.
pub fn parse_polling_rate(response: &Response) -> Result<u32, HidError> {
    ensure_success(response, opcodes::POLLING_GET)?;
    let selector = response.byte(6).unwrap_or(0);
    hid_to_polling_rate(selector).ok_or_else(|| {
        HidError::UnexpectedResponse(format!("unknown polling selector 0x{selector:02X}"))
    })
}

/// Debounce readback at index 7.
pub fn parse_debounce(response: &Response) -> Result<u8, HidError> {
    ensure_success(response, opcodes::DEBOUNCE_GET)?;
    response
        .byte(7)
        .filter(|value| *value <= 50)
        .ok_or_else(|| HidError::UnexpectedResponse("missing debounce value".into()))
}

/// Lift-off distance readback: `1` or `2` mm.
pub fn parse_lod(response: &Response) -> Result<u8, HidError> {
    ensure_success(response, opcodes::LOD_GET)?;
    match (response.byte(6), response.byte(7)) {
        (Some(1 | 2), _) => Ok(response.byte(6).unwrap()),
        (_, Some(1 | 2)) => Ok(response.byte(7).unwrap()),
        _ => Err(HidError::UnexpectedResponse(
            "missing lift-off distance".into(),
        )),
    }
}

/// Map a polling rate to the Sinowealth HID selector.
pub fn polling_rate_to_hid(rate: u32) -> Option<u8> {
    match rate {
        125 => Some(0x08),
        250 => Some(0x04),
        500 => Some(0x02),
        1000 => Some(0x01),
        2000 => Some(0x20),
        4000 => Some(0x40),
        8000 => Some(0x80),
        _ => None,
    }
}

pub fn hid_to_polling_rate(selector: u8) -> Option<u32> {
    match selector {
        0x08 => Some(125),
        0x04 => Some(250),
        0x02 => Some(500),
        0x01 => Some(1000),
        0x20 => Some(2000),
        0x40 => Some(4000),
        0x80 => Some(8000),
        _ => None,
    }
}

/// Animation speed byte from the UI rate (1-100), matching Glorious CORE.
pub fn rgb_speed(rate: u8, effect: u8) -> u8 {
    let rate = rate.clamp(1, 100) as i32;
    let speed = if effect == 6 || effect == 7 {
        (105 - rate) * 2
    } else {
        (105 - rate) / 5
    };
    speed.clamp(0, 255) as u8
}

/// 0-100 % to the firmware's 0-255 brightness scale.
pub fn brightness_to_byte(percent: u8) -> u8 {
    ((percent.min(100) as u16 * 255) / 100) as u8
}

// ---------------------------------------------------------------------------
// Device handle
// ---------------------------------------------------------------------------

pub struct SinowealthDevice {
    dev: HidDevice,
    endpoint: Endpoint,
    kind: ConnectionKind,
    delay: Duration,
    new_performance_layout: bool,
}

impl SinowealthDevice {
    /// Detect the vendor interface, open it and return a handle.
    pub fn open(
        api: &HidApi,
        entry: &DeviceEntry,
        connection: &Connection,
        delay: Duration,
    ) -> Result<Self, HidError> {
        let (vid, pid) = connection
            .vid_pid()
            .ok_or_else(|| HidError::InvalidInput(format!("bad VID/PID in {connection:?}")))?;
        let endpoint = hid::select_vendor_endpoint(api, vid, pid, entry.vendor_usage())?;
        let dev = hid::open_vendor_device(api, &endpoint)?;
        Ok(Self {
            dev,
            endpoint,
            kind: ConnectionKind::from_label(&connection.kind),
            delay,
            new_performance_layout: entry.uses_new_performance_layout(),
        })
    }

    pub fn endpoint(&self) -> &Endpoint {
        &self.endpoint
    }

    pub fn connection_kind(&self) -> ConnectionKind {
        self.kind
    }

    fn transact(
        &self,
        target: u8,
        category: u8,
        opcode: u8,
        payload: &[u8],
        delay: Duration,
        attempts: usize,
    ) -> Result<Response, HidError> {
        let mut last_error: Option<HidError> = None;

        for attempt in 0..attempts {
            let request = build_request(target, category, opcode, payload);
            self.dev.send_feature_report(&request)?;
            thread::sleep(delay);

            let mut raw = vec![0u8; REPORT_LEN];
            let read = self.dev.get_feature_report(&mut raw)?;
            raw.truncate(read);
            let response = Response::normalize(&raw, opcode);

            if response.echo() != Some(opcode) {
                last_error = Some(HidError::UnexpectedResponse(format!(
                    "opcode echo mismatch for 0x{opcode:02X}: got {:?} (attempt {}/{attempts})",
                    response.echo(),
                    attempt + 1
                )));
                continue;
            }

            match response.status() {
                DeviceStatus::Success => return Ok(response),
                DeviceStatus::Unsupported => {
                    return Err(HidError::Protocol {
                        status: response.status_byte(),
                        detail: format!("opcode 0x{opcode:02X} is not supported"),
                    });
                }
                status => {
                    last_error = Some(HidError::Protocol {
                        status: response.status_byte(),
                        detail: format!("{} (attempt {}/{attempts})", status.as_str(), attempt + 1),
                    });
                    // Busy / RF retry needs a little breathing room.
                    thread::sleep(Duration::from_millis(40));
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            HidError::UnexpectedResponse(format!("no response for opcode 0x{opcode:02X}"))
        }))
    }

    /// Fire-and-forget write. Configuration writes are not acknowledged by the
    /// original driver either; explicit getters are used for verification.
    fn write(&self, target: u8, category: u8, opcode: u8, payload: &[u8]) -> Result<(), HidError> {
        let request = build_request(target, category, opcode, payload);
        self.dev.send_feature_report(&request)?;
        thread::sleep(self.delay);
        Ok(())
    }

    fn read_firmware(&self, target: u8, delay: Duration) -> Result<String, HidError> {
        let response = self.transact(
            target,
            CATEGORY_SYSTEM,
            opcodes::FIRMWARE,
            &[0, 0, 0],
            delay,
            3,
        )?;
        parse_firmware(&response)
    }

    /// Dongle and mouse firmware. The mouse is queried best-effort because it
    /// can be asleep while the receiver still answers.
    pub fn firmware(&self) -> Result<FirmwareInfo, HidError> {
        let mut info = FirmwareInfo::default();

        match self.kind {
            ConnectionKind::Wired | ConnectionKind::Bluetooth => {
                info.mouse = Some(self.read_firmware(TARGET_MOUSE, Duration::from_millis(100))?);
            }
            ConnectionKind::Dongle | ConnectionKind::Unknown => {
                info.dongle = Some(self.read_firmware(TARGET_DONGLE, Duration::from_millis(120))?);
                if let Ok(version) = self.read_firmware(TARGET_MOUSE, Duration::from_millis(100)) {
                    info.mouse = Some(version);
                }
            }
        }

        info.display = info
            .dongle
            .clone()
            .or_else(|| info.mouse.clone())
            .unwrap_or_default();
        Ok(info)
    }

    pub fn battery(&self) -> Result<BatteryInfo, HidError> {
        let response = self.transact(
            TARGET_MOUSE,
            CATEGORY_SYSTEM,
            opcodes::BATTERY,
            &[0, 0],
            self.delay,
            3,
        )?;
        parse_battery(&response)
    }

    pub fn active_profile(&self) -> Result<u8, HidError> {
        let response = self.transact(
            TARGET_MOUSE,
            CATEGORY_SYSTEM,
            opcodes::PROFILE_GET,
            &[0],
            self.delay,
            3,
        )?;
        parse_profile(&response)
    }

    pub fn sleep_seconds(&self) -> Result<Option<u32>, HidError> {
        let response = self.transact(
            TARGET_MOUSE,
            CATEGORY_SYSTEM,
            opcodes::SLEEP_GET,
            &[0, 0],
            self.delay,
            3,
        )?;
        parse_sleep_seconds(&response)
    }

    pub fn dpi_info(&self, profile: u8) -> Result<DpiInfo, HidError> {
        let response = self.transact(
            TARGET_MOUSE,
            CATEGORY_SENSOR,
            opcodes::DPI_VALUES_GET,
            &[profile, 0],
            self.delay,
            3,
        )?;
        parse_dpi_info(&response)
    }

    pub fn dpi_stages(&self, profile: u8) -> Result<Vec<u32>, HidError> {
        self.dpi_info(profile).map(|info| info.stages)
    }

    pub fn active_dpi_stage(&self, profile: u8) -> Result<u8, HidError> {
        let response = self.transact(
            TARGET_MOUSE,
            CATEGORY_SENSOR,
            opcodes::ACTIVE_DPI_GET,
            &[profile, 0],
            self.delay,
            3,
        )?;
        parse_active_dpi_stage(&response)
    }

    pub fn polling_rate(&self, profile: u8) -> Result<u32, HidError> {
        let response = self.transact(
            TARGET_MOUSE,
            CATEGORY_SENSOR,
            opcodes::POLLING_GET,
            &[profile, 0],
            self.delay,
            3,
        )?;
        parse_polling_rate(&response)
    }

    pub fn debounce(&self, profile: u8) -> Result<u8, HidError> {
        let response = self.transact(
            TARGET_MOUSE,
            CATEGORY_SYSTEM,
            opcodes::DEBOUNCE_GET,
            &[profile, 0],
            self.delay,
            3,
        )?;
        parse_debounce(&response)
    }

    pub fn lod(&self, profile: u8) -> Result<u8, HidError> {
        let response = self.transact(
            TARGET_MOUSE,
            CATEGORY_SENSOR,
            opcodes::LOD_GET,
            &[profile, 0],
            self.delay,
            3,
        )?;
        parse_lod(&response)
    }

    /// Full settings snapshot. Individual getters are best-effort so a single
    /// unsupported command cannot blank the whole response.
    pub fn settings(&self) -> Result<SettingsView, HidError> {
        let profile = self.active_profile()?;
        let mut settings = SettingsView {
            active_profile: profile,
            ..Default::default()
        };

        match self.dpi_info(profile) {
            Ok(info) => {
                settings.dpi_stage_count = info.stage_count;
                settings.dpi_stages = info.stages;
            }
            Err(_) => {
                settings.dpi_stage_count = 0;
            }
        }
        settings.active_dpi_stage = self.active_dpi_stage(profile).unwrap_or(1);
        settings.polling_rate = self.polling_rate(profile).unwrap_or(1000);
        settings.debounce_ms = self.debounce(profile).unwrap_or(10);
        settings.lod = self.lod(profile).unwrap_or(1);
        settings.sleep_seconds = self.sleep_seconds().unwrap_or(None);

        Ok(settings)
    }

    /// Apply a validated patch. Returns the list of settings that were written.
    pub fn apply(&self, patch: &SettingsPatch) -> Result<Vec<String>, HidError> {
        patch.validate().map_err(HidError::InvalidInput)?;

        let mut applied = Vec::new();
        let mut profile = patch
            .profile
            .or_else(|| self.active_profile().ok())
            .unwrap_or(1);

        if let Some(requested) = patch.profile {
            self.set_profile(requested)?;
            profile = requested;
            applied.push("profile".to_string());
        }

        if let Some(stages) = &patch.dpi_stages {
            self.set_dpi_stages(profile, stages)?;
            applied.push("dpi_stages".to_string());
        }

        if let Some(colors) = &patch.dpi_colors {
            let mut rgb_colors = Vec::with_capacity(colors.len());
            for c in colors {
                let rgb = crate::types::parse_hex_color(c).ok_or_else(|| {
                    HidError::InvalidInput(format!("invalid hex color '{c}' in dpi_colors"))
                })?;
                rgb_colors.push(rgb);
            }
            self.set_dpi_colors(profile, &rgb_colors)?;
            applied.push("dpi_colors".to_string());
        }

        if let Some(stage) = patch.active_dpi_stage {
            self.set_active_dpi_stage(profile, stage)?;
            applied.push("active_dpi_stage".to_string());
        }

        if let Some(rate) = patch.polling_rate {
            self.set_polling_rate(profile, rate)?;
            applied.push("polling_rate".to_string());
        }

        if let Some(debounce) = patch.debounce_ms {
            self.set_debounce(profile, debounce)?;
            applied.push("debounce_ms".to_string());
        }

        if let Some(lod) = patch.lod {
            self.set_lod(profile, lod)?;
            applied.push("lod".to_string());
        }

        if let Some(seconds) = patch.sleep_seconds {
            self.set_sleep_seconds(seconds)?;
            applied.push("sleep_seconds".to_string());
        }

        if let Some(rgb) = &patch.rgb {
            self.set_rgb(profile, rgb)?;
            applied.push("rgb".to_string());
        }

        Ok(applied)
    }

    pub fn set_profile(&self, profile: u8) -> Result<(), HidError> {
        self.write(
            TARGET_MOUSE,
            CATEGORY_SYSTEM,
            opcodes::PROFILE_SET,
            &[profile],
        )
    }

    pub fn set_sleep_seconds(&self, seconds: u32) -> Result<(), HidError> {
        let encoded = if seconds == 0 {
            0xFFFFu16
        } else {
            seconds.min(u16::MAX as u32) as u16
        };
        self.write(
            TARGET_MOUSE,
            CATEGORY_SYSTEM,
            opcodes::SLEEP_SET,
            &[(encoded >> 8) as u8, (encoded & 0xFF) as u8],
        )
    }

    pub fn set_debounce(&self, profile: u8, millis: u8) -> Result<(), HidError> {
        self.write(
            TARGET_MOUSE,
            CATEGORY_SYSTEM,
            opcodes::DEBOUNCE_SET,
            &[profile, millis],
        )
    }

    pub fn set_dpi_stages(&self, profile: u8, stages: &[u32]) -> Result<(), HidError> {
        if stages.is_empty() || stages.len() > 8 {
            return Err(HidError::InvalidInput(format!(
                "dpi_stages must contain 1-8 entries, got {}",
                stages.len()
            )));
        }

        let mut payload = Vec::with_capacity(2 + stages.len() * 4);
        payload.push(profile);
        payload.push(stages.len() as u8);
        for dpi in stages {
            if !(100..=30000).contains(dpi) {
                return Err(HidError::InvalidInput(format!(
                    "DPI {dpi} outside 100-30000"
                )));
            }
            let hi = (dpi >> 8) as u8;
            let lo = (dpi & 0xFF) as u8;
            // X and Y use the same resolution.
            payload.extend_from_slice(&[hi, lo, hi, lo]);
        }

        self.write(
            TARGET_MOUSE,
            CATEGORY_SENSOR,
            opcodes::DPI_VALUES_SET,
            &payload,
        )
    }

    pub fn set_dpi_colors(&self, profile: u8, colors: &[[u8; 3]]) -> Result<(), HidError> {
        if colors.is_empty() || colors.len() > 8 {
            return Err(HidError::InvalidInput(format!(
                "dpi_colors must contain 1-8 entries, got {}",
                colors.len()
            )));
        }

        let mut payload = Vec::with_capacity(1 + colors.len() * 3);
        payload.push(profile);
        for color in colors {
            payload.extend_from_slice(color);
        }

        self.write(
            TARGET_MOUSE,
            CATEGORY_RGB,
            opcodes::RGB_DPI_COLOR_SET,
            &payload,
        )
    }

    pub fn set_active_dpi_stage(&self, profile: u8, stage: u8) -> Result<(), HidError> {
        if !(1..=8).contains(&stage) {
            return Err(HidError::InvalidInput(format!(
                "active DPI stage must be 1-8, got {stage}"
            )));
        }
        self.write(
            TARGET_MOUSE,
            CATEGORY_SENSOR,
            opcodes::ACTIVE_DPI_SET,
            &[profile, stage],
        )
    }

    pub fn set_polling_rate(&self, profile: u8, rate: u32) -> Result<(), HidError> {
        let selector = polling_rate_to_hid(rate)
            .ok_or_else(|| HidError::InvalidInput(format!("unsupported polling rate {rate}")))?;

        if self.new_performance_layout {
            self.write(
                TARGET_MOUSE,
                CATEGORY_SENSOR,
                opcodes::POLLING_SET_V2,
                &[profile, selector],
            )
        } else {
            self.write(
                TARGET_MOUSE,
                CATEGORY_SENSOR,
                opcodes::POLLING_SET,
                &[selector],
            )
        }
    }

    pub fn set_lod(&self, profile: u8, lod: u8) -> Result<(), HidError> {
        if lod != 1 && lod != 2 {
            return Err(HidError::InvalidInput(format!(
                "lift-off distance must be 1 or 2 mm, got {lod}"
            )));
        }

        if self.new_performance_layout {
            self.write(
                TARGET_MOUSE,
                CATEGORY_SENSOR,
                opcodes::LOD_SET,
                &[profile, lod],
            )
        } else {
            self.write(TARGET_MOUSE, CATEGORY_SENSOR, opcodes::LOD_SET, &[lod])
        }
    }

    pub fn set_rgb(&self, profile: u8, rgb: &RgbPatch) -> Result<(), HidError> {
        let effect = rgb.effect.resolve().map_err(HidError::InvalidInput)?;
        let speed = rgb_speed(rgb.rate, effect);

        let mut payload = Vec::with_capacity(8);
        payload.push(profile);
        payload.push(0xFF); // all zones
        payload.push(effect);
        payload.push(0x00); // direction
        payload.push(speed);
        if effect == 2 {
            // Seamless breathing ignores the configured colors and uses red.
            payload.extend_from_slice(&[0xFF, 0x00, 0x00]);
        } else {
            payload.extend_from_slice(&rgb.color);
        }

        self.write(
            TARGET_MOUSE,
            CATEGORY_RGB,
            opcodes::RGB_EFFECT_SET,
            &payload,
        )?;

        let wired = brightness_to_byte(rgb.brightness);
        let wireless = brightness_to_byte(rgb.wireless_brightness.unwrap_or(rgb.brightness));
        self.write(
            TARGET_MOUSE,
            CATEGORY_RGB,
            opcodes::RGB_BRIGHTNESS_SET,
            &[1, wired],
        )?;
        self.write(
            TARGET_MOUSE,
            CATEGORY_RGB,
            opcodes::RGB_BRIGHTNESS_SET,
            &[0, wireless],
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw_report(body: &[u8]) -> Vec<u8> {
        let mut raw = vec![0u8; REPORT_LEN];
        for (index, byte) in body.iter().enumerate() {
            if index < raw.len() {
                raw[index] = *byte;
            }
        }
        raw
    }

    /// Captured from a Model O Wireless dongle (0x258A:0x2022).
    fn dongle_firmware_report() -> Vec<u8> {
        raw_report(&[
            0x00, 0xA1, 0x00, 0x00, 0x04, 0x00, 0x81, 0x00, 0x03, 0x08, 0x01, 0x20, 0x22,
        ])
    }

    fn battery_report() -> Vec<u8> {
        raw_report(&[0x00, 0xA1, 0x00, 0x02, 0x02, 0x00, 0x83, 0x00, 0x55])
    }

    #[test]
    fn builds_65_byte_request() {
        let request = build_request(TARGET_MOUSE, CATEGORY_SENSOR, 0x81, &[1, 2]);
        assert_eq!(request.len(), 65);
        assert_eq!(request[0], 0x00);
        assert_eq!(request[3], 0x02);
        assert_eq!(request[4], 0x02);
        assert_eq!(request[5], 0x01);
        assert_eq!(request[6], 0x81);
        assert_eq!(&request[7..9], &[1, 2]);
        assert!(request[9..].iter().all(|b| *b == 0));
    }

    #[test]
    fn truncates_payload_to_report_bounds() {
        let payload = vec![0xAA; 100];
        let request = build_request(TARGET_MOUSE, CATEGORY_RGB, 0x00, &payload);
        assert_eq!(&request[7..], &payload[..58]);
        assert_eq!(request[4], 100);
    }

    #[test]
    fn normalizes_report_with_id() {
        let response = Response::normalize(&dongle_firmware_report(), 0x81);
        assert_eq!(response.as_slice().len(), 64);
        assert_eq!(response.status(), DeviceStatus::Success);
        assert_eq!(response.echo(), Some(0x81));
        assert_eq!(response.byte(6), Some(0x00));
        assert_eq!(response.byte(7), Some(0x03));
    }

    #[test]
    fn normalizes_report_without_id() {
        let raw = &dongle_firmware_report()[1..];
        let response = Response::normalize(raw, 0x81);
        assert_eq!(response.as_slice().len(), 64);
        assert_eq!(response.status(), DeviceStatus::Success);
        assert_eq!(response.echo(), Some(0x81));
    }

    #[test]
    fn parses_dongle_firmware_as_0_3_8_1() {
        let response = Response::normalize(&dongle_firmware_report(), 0x81);
        assert_eq!(parse_firmware(&response).unwrap(), "0.3.8.1");
    }

    #[test]
    fn parses_battery_percentage_and_charging() {
        let response = Response::normalize(&battery_report(), 0x83);
        let battery = parse_battery(&response).unwrap();
        assert_eq!(battery.percentage, 0x55);
        assert!(!battery.charging);
        assert_eq!(battery.status, "ok");
    }

    #[test]
    fn promotes_zero_percent_to_one() {
        let raw = raw_report(&[0x00, 0xA1, 0x00, 0x02, 0x02, 0x00, 0x83, 0x01, 0x00]);
        let response = Response::normalize(&raw, 0x83);
        let battery = parse_battery(&response).unwrap();
        assert_eq!(battery.percentage, 1);
        assert!(battery.charging);
    }

    #[test]
    fn rejects_battery_out_of_range() {
        let raw = raw_report(&[0x00, 0xA1, 0x00, 0x02, 0x02, 0x00, 0x83, 0x00, 0xFE]);
        let response = Response::normalize(&raw, 0x83);
        assert!(parse_battery(&response).is_err());
    }

    #[test]
    fn parses_dpi_stages() {
        let mut body = vec![0x00, 0xA1, 0x00, 0x02, 0x0A, 0x00, 0x81, 0x01, 0x02];
        body.extend_from_slice(&[0x03, 0x20, 0x03, 0x20]); // 800
        body.extend_from_slice(&[0x06, 0x40, 0x06, 0x40]); // 1600
        let response = Response::normalize(&raw_report(&body), 0x81);
        assert_eq!(parse_dpi_stages(&response).unwrap(), vec![800, 1600]);
    }

    #[test]
    fn keeps_stage_count_when_resolutions_are_zeroed() {
        // Captured from Model O Wireless: firmware reports 4 stages but zeroed
        // resolutions (Glorious CORE keeps the database values instead).
        let body = [0x00, 0xA1, 0x00, 0x02, 0x12, 0x01, 0x81, 0x01, 0x04];
        let response = Response::normalize(&raw_report(&body), 0x81);
        let info = parse_dpi_info(&response).unwrap();
        assert_eq!(info.stage_count, 4);
        assert!(info.stages.is_empty());
    }

    #[test]
    fn parses_active_dpi_stage() {
        let body = [0x00, 0xA1, 0x00, 0x02, 0x02, 0x00, 0x82, 0x01, 0x03];
        let response = Response::normalize(&raw_report(&body), 0x82);
        assert_eq!(parse_active_dpi_stage(&response).unwrap(), 3);
    }

    #[test]
    fn parses_polling_rate() {
        let body = [0x00, 0xA1, 0x00, 0x02, 0x02, 0x00, 0x80, 0x01];
        let response = Response::normalize(&raw_report(&body), 0x80);
        assert_eq!(parse_polling_rate(&response).unwrap(), 1000);
    }

    #[test]
    fn parses_debounce_and_lod() {
        let debounce = Response::normalize(
            &raw_report(&[0x00, 0xA1, 0x00, 0x02, 0x02, 0x00, 0x88, 0x01, 0x04]),
            0x88,
        );
        assert_eq!(parse_debounce(&debounce).unwrap(), 4);

        let lod = Response::normalize(
            &raw_report(&[0x00, 0xA1, 0x00, 0x02, 0x02, 0x00, 0x88, 0x02, 0x00]),
            0x88,
        );
        assert_eq!(parse_lod(&lod).unwrap(), 2);

        let lod_fallback = Response::normalize(
            &raw_report(&[0x00, 0xA1, 0x00, 0x02, 0x02, 0x00, 0x88, 0x00, 0x02]),
            0x88,
        );
        assert_eq!(parse_lod(&lod_fallback).unwrap(), 2);
    }

    #[test]
    fn parses_sleep_timer() {
        let body = [0x00, 0xA1, 0x00, 0x02, 0x02, 0x00, 0x87, 0x02, 0x58];
        let response = Response::normalize(&raw_report(&body), 0x87);
        assert_eq!(parse_sleep_seconds(&response).unwrap(), Some(600));

        let disabled = Response::normalize(
            &raw_report(&[0x00, 0xA1, 0x00, 0x02, 0x02, 0x00, 0x87, 0xFF, 0xFF]),
            0x87,
        );
        assert_eq!(parse_sleep_seconds(&disabled).unwrap(), None);
    }

    #[test]
    fn maps_polling_rates_both_ways() {
        for (rate, selector) in [
            (125, 0x08),
            (250, 0x04),
            (500, 0x02),
            (1000, 0x01),
            (2000, 0x20),
            (4000, 0x40),
            (8000, 0x80),
        ] {
            assert_eq!(polling_rate_to_hid(rate), Some(selector));
            assert_eq!(hid_to_polling_rate(selector), Some(rate));
        }
        assert_eq!(polling_rate_to_hid(333), None);
    }

    #[test]
    fn computes_rgb_speed_like_glorious_core() {
        assert_eq!(rgb_speed(100, 1), 1);
        assert_eq!(rgb_speed(50, 4), 11);
        assert_eq!(rgb_speed(50, 7), 110);
    }

    #[test]
    fn maps_brightness_percent() {
        assert_eq!(brightness_to_byte(0), 0);
        assert_eq!(brightness_to_byte(100), 255);
        assert_eq!(brightness_to_byte(50), 127);
    }

    #[test]
    fn reports_protocol_status_errors() {
        let raw = raw_report(&[0x00, 0xA4, 0x00, 0x02, 0x02, 0x00, 0x83, 0x00, 0x00]);
        let response = Response::normalize(&raw, 0x83);
        assert_eq!(response.status(), DeviceStatus::NoDevice);
        assert_eq!(response.status().as_str(), "asleep");
        let err = parse_battery(&response).unwrap_err();
        assert!(matches!(err, HidError::Protocol { status: 0xA4, .. }));
    }

    #[test]
    fn builds_dpi_color_request() {
        let colors = [[255, 255, 0], [0, 0, 255], [255, 0, 0], [0, 255, 0]];
        let mut payload = vec![1];
        for c in &colors {
            payload.extend_from_slice(c);
        }
        let request = build_request(TARGET_MOUSE, CATEGORY_RGB, opcodes::RGB_DPI_COLOR_SET, &payload);
        assert_eq!(request[0], 0x00);
        assert_eq!(request[3], TARGET_MOUSE);
        assert_eq!(request[4], 13);
        assert_eq!(request[5], CATEGORY_RGB);
        assert_eq!(request[6], opcodes::RGB_DPI_COLOR_SET);
        assert_eq!(request[7], 1); // profile
        assert_eq!(&request[8..11], &[255, 255, 0]);
        assert_eq!(&request[11..14], &[0, 0, 255]);
        assert_eq!(&request[14..17], &[255, 0, 0]);
        assert_eq!(&request[17..20], &[0, 255, 0]);
    }
}
