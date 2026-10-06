// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The commands the webview can call.
//!
//! Every command is registered by name in `build.rs`, which auto-generates the
//! `allow-*` and `deny-*` permissions that a capability must reference. The
//! JavaScript wrapper for each one lives in `guest-js/index.ts`.

use tauri::{command, AppHandle, Runtime};

use crate::models::{BrightnessPayload, Diagnostics, MonitorInfo};
use crate::BrightnessExt;
use crate::Result;

/// Returns every display whose brightness can be controlled.
///
/// The list is recomputed on each call so that hot-plugged monitors are picked
/// up without restarting the application.
#[command]
pub(crate) async fn list_monitors<R: Runtime>(app: AppHandle<R>) -> Result<Vec<MonitorInfo>> {
    Ok(app.brightness().list_monitors())
}

/// Reads the brightness of one display.
#[command]
pub(crate) async fn get_brightness<R: Runtime>(
    app: AppHandle<R>,
    id: String,
) -> Result<BrightnessPayload> {
    app.brightness().get_brightness(&id)
}

/// Sets the brightness of one display.
#[command]
pub(crate) async fn set_brightness<R: Runtime>(
    app: AppHandle<R>,
    id: String,
    value: u8,
) -> Result<()> {
    app.brightness().set_brightness(&id, value)
}

/// Explains why a display may not have been detected.
#[command]
pub(crate) async fn diagnose<R: Runtime>(app: AppHandle<R>) -> Result<Diagnostics> {
    Ok(app.brightness().diagnose())
}
