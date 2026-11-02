//! HID transport: endpoint discovery, vendor interface selection and the
//! protocol implementations.
//!
//! Linux exposes each HID collection as its own `/dev/hidraw*` node. A Glorious
//! mouse therefore shows up as multiple nodes with the same VID/PID; only the
//! vendor collection accepts the 65-byte configuration feature report:
//!
//! * Model O Wireless dongle: interface 2, usage page `0xFFFF`, usage `0x00`.
//! * Model I 2 / PixArt family: usage page `0xFF00`, usage `0x01` (scaffold).

pub mod pixart;
pub mod sinowealth;

use std::ffi::CString;
use std::fmt;
use std::time::Duration;

use hidapi::{HidApi, HidDevice};
use serde::Serialize;

pub use pixart::PixArtDevice;
pub use sinowealth::SinowealthDevice;

/// Feature report ID used by the Sinowealth vendor protocol.
pub const REPORT_ID: u8 = 0x00;
/// Total feature report length including the leading report ID byte.
pub const REPORT_LEN: usize = 65;
/// Default delay between `HIDIOCSFEATURE` and `HIDIOCGFEATURE`.
///
/// The wireless dongle needs 50-100 ms to relay a report to the mouse over RF;
/// reading earlier returns `0xA4` (device not found).
pub const DEFAULT_DELAY: Duration = Duration::from_millis(80);

/// Physical link a device was found on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionKind {
    Wired,
    Dongle,
    Bluetooth,
    Unknown,
}

impl ConnectionKind {
    pub fn from_label(label: &str) -> Self {
        match label.to_ascii_lowercase().as_str() {
            "usb" | "wired" => Self::Wired,
            "dongle" | "wireless" => Self::Dongle,
            "bluetooth" | "bt" => Self::Bluetooth,
            _ => Self::Unknown,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Wired => "wired",
            Self::Dongle => "dongle",
            Self::Bluetooth => "bluetooth",
            Self::Unknown => "unknown",
        }
    }
}

/// Status codes returned in byte 0 of the normalized response payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceStatus {
    Success,
    Busy,
    Failure,
    Unsupported,
    NoDevice,
    Unknown(u8),
}

impl DeviceStatus {
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0xA1 => Self::Success,
            0xA0 => Self::Busy,
            0xA2 => Self::Failure,
            0xA3 => Self::Unsupported,
            0xA4 => Self::NoDevice,
            other => Self::Unknown(other),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Success => "ok",
            Self::Busy => "busy",
            Self::Failure => "failure",
            Self::Unsupported => "unsupported",
            Self::NoDevice => "asleep",
            Self::Unknown(_) => "unknown",
        }
    }

    pub fn is_success(self) -> bool {
        matches!(self, Self::Success)
    }
}

/// One HID node belonging to one device.
#[derive(Debug, Clone)]
pub struct Endpoint {
    pub vid: u16,
    pub pid: u16,
    pub interface_number: i32,
    pub usage_page: u16,
    pub usage: u16,
    pub product: Option<String>,
    pub manufacturer: Option<String>,
    pub serial: Option<String>,
    pub path: CString,
}

impl Endpoint {
    pub fn path_display(&self) -> String {
        self.path.to_string_lossy().into_owned()
    }
}

impl fmt::Display for Endpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:04X}:{:04X} iface={} usage={:04X}:{:04X} {}",
            self.vid,
            self.pid,
            self.interface_number,
            self.usage_page,
            self.usage,
            self.path_display()
        )
    }
}

/// Enumerate every HID node. Cheap compared to opening a handle.
pub fn enumerate(api: &HidApi) -> Vec<Endpoint> {
    api.device_list()
        .map(|info| Endpoint {
            vid: info.vendor_id(),
            pid: info.product_id(),
            interface_number: info.interface_number(),
            usage_page: info.usage_page(),
            usage: info.usage(),
            product: info.product_string().map(str::to_owned),
            manufacturer: info.manufacturer_string().map(str::to_owned),
            serial: info.serial_number().map(str::to_owned),
            path: info.path().to_owned(),
        })
        .collect()
}

/// All HID nodes that share a VID/PID pair.
pub fn endpoints_for(api: &HidApi, vid: u16, pid: u16) -> Vec<Endpoint> {
    let mut endpoints: Vec<Endpoint> = enumerate(api)
        .into_iter()
        .filter(|e| e.vid == vid && e.pid == pid)
        .collect();
    endpoints.sort_by_key(|e| (e.interface_number, e.usage_page, e.usage));
    endpoints
}

/// Pick the node that carries vendor feature reports.
///
/// Selection order (mirrors the Linux driver used by Glorious CORE):
/// 1. exact usage page/usage from `data/glorious_devices.json`;
/// 2. `interface_number == 2` (Model O Wireless dongle on Linux);
/// 3. usage page `0xFFFF` / usage `0x00`;
/// 4. anything with the expected usage page;
/// 5. the last node returned by the kernel.
pub fn select_vendor_endpoint(
    api: &HidApi,
    vid: u16,
    pid: u16,
    desired: Option<(u16, u16)>,
) -> Result<Endpoint, HidError> {
    let candidates = endpoints_for(api, vid, pid);
    choose_endpoint(&candidates, desired).ok_or(HidError::DeviceNotFound { vid, pid })
}

/// Choose the vendor feature-report node from a pre-enumerated candidate list.
pub fn choose_endpoint(candidates: &[Endpoint], desired: Option<(u16, u16)>) -> Option<Endpoint> {
    if candidates.is_empty() {
        return None;
    }

    if let Some((page, usage)) = desired {
        if let Some(found) = candidates
            .iter()
            .find(|e| e.usage_page == page && e.usage == usage)
        {
            return Some(found.clone());
        }
    }

    if let Some(found) = candidates
        .iter()
        .find(|e| e.interface_number == 2 && e.usage_page == 0xFFFF)
        .or_else(|| candidates.iter().find(|e| e.interface_number == 2))
    {
        return Some(found.clone());
    }

    if let Some(found) = candidates
        .iter()
        .find(|e| e.usage_page == 0xFFFF && e.usage == 0x00)
    {
        return Some(found.clone());
    }

    let expected_page = desired.map(|(page, _)| page);
    if let Some(page) = expected_page {
        if let Some(found) = candidates.iter().find(|e| e.usage_page == page) {
            return Some(found.clone());
        }
    }

    candidates.last().cloned()
}

/// Open the vendor feature-report handle for a device.
pub fn open_vendor_device(api: &HidApi, endpoint: &Endpoint) -> Result<HidDevice, HidError> {
    api.open_path(endpoint.path.as_c_str())
        .map_err(HidError::Backend)
}

#[derive(Debug)]
pub enum HidError {
    Backend(hidapi::HidError),
    DeviceNotFound {
        vid: u16,
        pid: u16,
    },
    VendorInterfaceNotFound {
        vid: u16,
        pid: u16,
        desired: Option<(u16, u16)>,
    },
    Protocol {
        status: u8,
        detail: String,
    },
    UnexpectedResponse(String),
    Unsupported {
        family: &'static str,
        detail: String,
    },
    InvalidInput(String),
}

impl fmt::Display for HidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Backend(err) => write!(f, "HID backend error: {err}"),
            Self::DeviceNotFound { vid, pid } => {
                write!(f, "no HID device found for {vid:04X}:{pid:04X}")
            }
            Self::VendorInterfaceNotFound { vid, pid, desired } => match desired {
                Some((page, usage)) => write!(
                    f,
                    "no vendor interface {page:04X}:{usage:04X} for {vid:04X}:{pid:04X}"
                ),
                None => write!(f, "no vendor interface for {vid:04X}:{pid:04X}"),
            },
            Self::Protocol { status, detail } => {
                write!(f, "device returned 0x{status:02X} ({detail})")
            }
            Self::UnexpectedResponse(detail) => write!(f, "unexpected device response: {detail}"),
            Self::Unsupported { family, detail } => {
                write!(f, "protocol '{family}' is not implemented: {detail}")
            }
            Self::InvalidInput(detail) => write!(f, "invalid input: {detail}"),
        }
    }
}

impl std::error::Error for HidError {}

impl From<hidapi::HidError> for HidError {
    fn from(value: hidapi::HidError) -> Self {
        Self::Backend(value)
    }
}
