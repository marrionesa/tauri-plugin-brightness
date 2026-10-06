// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Control display brightness through DDC/CI and the Linux kernel backlight.
//!
//! The plugin is a thin layer over the
//! [`tauri-brightness-core`](https://crates.io/crates/tauri-brightness-core)
//! crate: it exposes the same operations as commands, adds the permission
//! definitions Tauri requires, and ships JavaScript bindings.
//!
//! # Setup
//!
//! ```rust,no_run
//! # fn main() {
//! tauri::Builder::default().plugin(tauri_plugin_brightness::init());
//! # }
//! ```
//!
//! The plugin can also be used directly from Rust, through the
//! [`BrightnessExt`] extension trait:
//!
//! ```rust,no_run
//! use tauri::Manager;
//! use tauri_plugin_brightness::BrightnessExt;
//!
//! # fn inspect(app: &tauri::AppHandle) {
//! let monitors = app.brightness().list_monitors();
//! println!("{} display(s) detected", monitors.len());
//! # }
//! ```
//!
//! # Permissions
//!
//! The commands are not reachable from the webview until they are allowed by a
//! capability. `default.toml` allows all of them, so the common case only needs
//! the plugin to be registered:
//!
//! ```json
//! {
//!   "permissions": ["brightness:default"]
//! }
//! ```
//!
//! Grant `brightness:allow-set-brightness` alone when an application should be
//! able to change brightness but not read it.
//!
//! # Linux permissions
//!
//! DDC/CI needs the `i2c-dev` kernel module and usually membership of the `i2c`
//! group. Internal panels need neither. When no display is detected, the
//! `diagnose` command reports exactly which piece is missing.

#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod commands;
mod error;

// The two implementations expose the same operations but are only ever
// compiled for their own platform family, which is why `cfg` is used here and
// not inside a single module.
#[cfg(desktop)]
mod desktop;
#[cfg(desktop)]
use desktop as platform;
#[cfg(mobile)]
mod mobile;
#[cfg(mobile)]
use mobile as platform;

/// Data types shared with the JavaScript bindings.
pub mod models;

pub use error::{Error, Result};
pub use models::{Backend, BrightnessPayload, Diagnostics, MonitorInfo};

use platform::Brightness;

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

/// Extends a Tauri manager with access to the brightness API.
///
/// The trait is implemented for every [`Manager`], so it is available on
/// [`AppHandle`](tauri::AppHandle), [`App`](tauri::App) and
/// [`Window`](tauri::Window).
///
/// # Example
///
/// ```rust,no_run
/// use tauri::Manager;
/// use tauri_plugin_brightness::BrightnessExt;
///
/// # fn run(app: tauri::AppHandle) {
/// let monitors = app.brightness().list_monitors();
/// # let _ = monitors;
/// # }
/// ```
pub trait BrightnessExt<R: Runtime> {
    /// Returns the brightness API handle.
    fn brightness(&self) -> &Brightness<R>;
}

impl<R: Runtime, T: Manager<R>> BrightnessExt<R> for T {
    fn brightness(&self) -> &Brightness<R> {
        self.state::<Brightness<R>>().inner()
    }
}

/// Initializes the plugin.
///
/// Registers the commands, manages the brightness state, and loads the global
/// API script so the bindings are reachable from `window.__TAURI__.brightness`
/// in applications built with `withGlobalTauri`.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("brightness")
        .invoke_handler(tauri::generate_handler![
            commands::list_monitors,
            commands::get_brightness,
            commands::set_brightness,
            commands::diagnose,
        ])
        .setup(|app, _api| {
            let brightness = Brightness::new(app.clone());
            app.manage(brightness);
            Ok(())
        })
        .build()
}
