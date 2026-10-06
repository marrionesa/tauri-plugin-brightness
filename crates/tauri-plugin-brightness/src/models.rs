// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Data types shared between the commands, the desktop implementation and the
//! JavaScript bindings.
//!
//! The core crate already models displays and diagnostics, so these types are
//! re-exported rather than duplicated, which keeps a single source of truth for
//! the JSON shape seen by the frontend.

pub use tauri_brightness_core::{Backend, Diagnostics, MonitorInfo};

use serde::{Deserialize, Serialize};

/// Response of the `get_brightness` command.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BrightnessPayload {
    /// The display the value belongs to.
    pub id: String,
    /// The current brightness on the display's own scale.
    pub brightness: u8,
    /// The highest value the display accepts.
    pub max: u8,
    /// The current brightness as a percentage of `max`.
    pub percent: u8,
}

impl BrightnessPayload {
    /// Builds the payload for a display.
    pub fn new(id: impl Into<String>, brightness: u8, max: u8) -> Self {
        let percent = tauri_brightness_core::percent(brightness, max);
        Self {
            id: id.into(),
            brightness,
            max,
            percent,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_payload_carries_the_percentage_as_well_as_the_raw_value() {
        let payload = BrightnessPayload::new("ddc-0", 128, 255);
        assert_eq!(payload.percent, 50);
        assert_eq!(payload.brightness, 128);
        assert_eq!(payload.max, 255);
    }

    #[test]
    fn the_payload_serializes_with_camel_case_keys() {
        let json = serde_json::to_string(&BrightnessPayload::new("ddc-0", 80, 100)).unwrap();
        assert!(json.contains("\"id\":\"ddc-0\""));
        assert!(json.contains("\"percent\":80"));
    }
}
