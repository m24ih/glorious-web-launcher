//! Scaffold for the PixArt / Glorious SDK protocol family.
//!
//! `data/glorious_devices.json` marks this family with vendor usage pages
//! `0xFF00` / `0xFF01` instead of the Sinowealth `0xFFFF` page. Products in this
//! group include:
//!
//! * Model I 2 Wireless (`0x093A:0x821A`, dongle `0x093A:0x821D`)
//! * Model O 2 Wireless (`0x093A:0x822A`, dongle `0x093A:0x822D`)
//!
//! On Windows these are driven by `GloriousMOISDK-*.node`, not the older
//! Sinowealth 65-byte feature report. The command framing, report ids and
//! battery/lighting opcodes have not been verified against hardware yet, so this
//! module deliberately opens and reports the interface without issuing
//! speculative writes.
//!
//! TODO(next protocol pass):
//! 1. Capture `HIDIOCGFEATURE` traffic from Glorious CORE v2 for firmware and
//!    battery queries on 0x093A:0x821A.
//! 2. Confirm whether commands are prefixed with a report id, a checksum byte,
//!    or a length/opcode header.
//! 3. Implement `battery`, `firmware`, DPI, polling and RGB using the verified
//!    framing, then flip `capabilities().supported` to `true`.

use hidapi::HidApi;

use crate::devices::{Connection, DeviceEntry};
use crate::hid::{self, ConnectionKind, Endpoint, HidError};
use crate::types::{BatteryInfo, FirmwareInfo, SettingsPatch};

/// What the scaffold currently knows about a PixArt device.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PixArtCapabilities {
    pub family: &'static str,
    pub supported: bool,
    pub features: Vec<&'static str>,
    pub note: &'static str,
}

pub struct PixArtDevice {
    endpoint: Endpoint,
    kind: ConnectionKind,
    model: String,
}

impl PixArtDevice {
    /// Detect and open the vendor interface. No feature report is sent.
    pub fn open(
        api: &HidApi,
        entry: &DeviceEntry,
        connection: &Connection,
    ) -> Result<Self, HidError> {
        let (vid, pid) = connection
            .vid_pid()
            .ok_or_else(|| HidError::InvalidInput(format!("bad VID/PID in {connection:?}")))?;
        let endpoint = hid::select_vendor_endpoint(api, vid, pid, entry.vendor_usage())?;

        // Keep the handle validation cheap: opening catches permission errors and
        // confirms the node still exists, but we do not talk the unverified
        // protocol yet.
        hid::open_vendor_device(api, &endpoint)?;

        Ok(Self {
            endpoint,
            kind: ConnectionKind::from_label(&connection.kind),
            model: entry.name.clone(),
        })
    }

    pub fn endpoint(&self) -> &Endpoint {
        &self.endpoint
    }

    pub fn connection_kind(&self) -> ConnectionKind {
        self.kind
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn capabilities(&self) -> PixArtCapabilities {
        PixArtCapabilities {
            family: "pixart",
            supported: false,
            features: vec![],
            note: "Model I 2 / Model O 2 protocol capture pending; see module docs",
        }
    }

    pub fn battery(&self) -> Result<BatteryInfo, HidError> {
        Err(self.unsupported("battery"))
    }

    pub fn firmware(&self) -> Result<FirmwareInfo, HidError> {
        Err(self.unsupported("firmware"))
    }

    pub fn apply(&self, _patch: &SettingsPatch) -> Result<Vec<String>, HidError> {
        Err(self.unsupported("apply"))
    }

    fn unsupported(&self, feature: &str) -> HidError {
        HidError::Unsupported {
            family: "pixart",
            detail: format!(
                "{feature} for {} is not implemented yet (usage page 0x{:04X})",
                self.model, self.endpoint.usage_page
            ),
        }
    }
}
