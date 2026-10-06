// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The mobile implementation.
//!
//! Display brightness is a desktop concept and neither Android nor iOS exposes
//! an equivalent that a Tauri application may drive without extra permissions
//! and platform specific UI. The plugin therefore registers on mobile, so that
//! the same `Cargo.toml` and the same capability compile everywhere, but its
//! commands return [`tauri_brightness_core::Error::NoDisplays`].
//!
//! Applications that need mobile screen brightness should implement it behind
//! `#[cfg(mobile)]` and keep this plugin for desktop targets.

use tauri::{AppHandle, Runtime};

use crate::models::{BrightnessPayload, Diagnostics, MonitorInfo};
use crate::Result;

/// The mobile handle stored as managed state by [`crate::init`].
pub struct Brightness<R: Runtime> {
    #[allow(dead_code)]
    app: AppHandle<R>,
}

impl<R: Runtime> Brightness<R> {
    /// Creates the handle. Called once, from the plugin's setup hook.
    pub fn new(app: AppHandle<R>) -> Self {
        Self { app }
    }

    /// Always empty on mobile.
    pub fn list_monitors(&self) -> Vec<MonitorInfo> {
        Vec::new()
    }

    /// Always fails on mobile.
    pub fn get_brightness(&self, _id: &str) -> Result<BrightnessPayload> {
        Err(unsupported().into())
    }

    /// Always fails on mobile.
    pub fn set_brightness(&self, _id: &str, _value: u8) -> Result<()> {
        Err(unsupported().into())
    }

    /// Reports that the platform is unsupported.
    pub fn diagnose(&self) -> Diagnostics {
        Diagnostics {
            i2c_dev_loaded: false,
            i2c_dev_sysfs: false,
            i2c_devices: Vec::new(),
            in_i2c_group: false,
            in_video_group: false,
            ddcutil_available: false,
            ddcutil_detect_output: String::new(),
            ddc_hi_count: 0,
            ddcutil_count: 0,
            backlight_count: 0,
            has_nvidia: false,
            suggestions: vec![
                "Display brightness control is only available on desktop platforms".to_string(),
            ],
        }
    }
}

/// The error returned by every command on mobile.
fn unsupported() -> tauri_brightness_core::Error {
    tauri_brightness_core::Error::Os(
        "display brightness control is not supported on this platform".to_string(),
    )
}
