// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The desktop implementation.
//!
//! There is deliberately no shared state: the core crate re-enumerates the
//! displays on every call, which costs a little more but means a display that
//! was plugged in after start-up is immediately addressable.

use tauri::{AppHandle, Runtime};

use crate::models::{BrightnessPayload, Diagnostics, MonitorInfo};
use crate::Result;

/// The desktop handle stored as managed state by [`crate::init`].
///
/// Retrieved through the [`crate::BrightnessExt`] extension trait rather than
/// instantiated directly by users.
pub struct Brightness<R: Runtime> {
    #[allow(dead_code)]
    app: AppHandle<R>,
}

impl<R: Runtime> Brightness<R> {
    /// Creates the handle. Called once, from the plugin's setup hook.
    pub fn new(app: AppHandle<R>) -> Self {
        Self { app }
    }

    /// Lists every display whose brightness can be controlled.
    pub fn list_monitors(&self) -> Vec<MonitorInfo> {
        tauri_brightness_core::list_monitors()
    }

    /// Reads the brightness of one display.
    pub fn get_brightness(&self, id: &str) -> Result<BrightnessPayload> {
        let max = self
            .list_monitors()
            .into_iter()
            .find(|monitor| monitor.id == id)
            .map(|monitor| monitor.max)
            .unwrap_or(100);
        let brightness = tauri_brightness_core::get_brightness(id)?;
        Ok(BrightnessPayload::new(id, brightness, max))
    }

    /// Sets the brightness of one display.
    pub fn set_brightness(&self, id: &str, value: u8) -> Result<()> {
        tauri_brightness_core::set_brightness(id, value)?;
        Ok(())
    }

    /// Explains why a display may not have been detected.
    pub fn diagnose(&self) -> Diagnostics {
        tauri_brightness_core::diagnose()
    }
}
