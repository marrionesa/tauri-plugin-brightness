//! Cross-platform display brightness control.
//!
//! This crate is the engine behind the
//! [`tauri-plugin-brightness`](https://crates.io/crates/tauri-plugin-brightness)
//! plugin, and it has no dependency on Tauri, so it can be used from any Rust
//! program.
//!
//! Three backends are supported and are selected automatically:
//!
//! | Backend | Mechanism | Notes |
//! | --- | --- | --- |
//! | [`Backend::Ddc`] | DDC/CI over I2C, in process | Fast, no external tools |
//! | [`Backend::Ddcutil`] | The `ddcutil` command line tool | NVIDIA, docks, USB-DDC |
//! | [`Backend::Backlight`] | Linux `/sys/class/backlight` | Laptop internal panels |
//!
//! A display is addressed by a [`MonitorId`], whose textual form is stable for
//! the lifetime of a boot, for example `ddc-0` or `backlight-intel_backlight`.
//!
//! # Example
//!
//! ```no_run
//! use tauri_brightness_core::{list_monitors, set_brightness};
//!
//! for monitor in list_monitors() {
//!     println!("{} is at {}%", monitor.name, monitor.brightness_percent());
//!     set_brightness(&monitor.id, 50)?;
//! }
//! # Ok::<(), tauri_brightness_core::Error>(())
//! ```
//!
//! # Permissions
//!
//! On Linux, external monitors reach DDC/CI through `/dev/i2c-*`, which requires
//! the `i2c-dev` kernel module and usually membership of the `i2c` group.
//! Internal panels are controlled without privileges through the `logind` D-Bus
//! interface. When nothing is detected, [`diagnose`] reports which of these is
//! missing.

#![deny(missing_docs)]
#![deny(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod error;
mod models;

mod backends;

pub use error::{Error, Result};
pub use models::{percent, raw_from_percent, Backend, Diagnostics, MonitorId, MonitorInfo};

use backends::{backlight, ddc, ddcutil, diagnostics};

/// Enumerates every display whose brightness can be controlled.
///
/// The DDC/CI backend is tried first because it stays in process. The
/// `ddcutil` backend is only consulted when the first one found nothing, since
/// it shares the same protocol but costs a process per call. Kernel backlight
/// devices are always appended, because laptop panels are not DDC/CI devices.
///
/// Returns an empty list, rather than an error, when no display is found:
/// callers that want an explanation should use [`diagnose`].
pub fn list_monitors() -> Vec<MonitorInfo> {
    let mut monitors = ddc::enumerate();

    if monitors.is_empty() {
        log::info!("the in-process DDC/CI backend found no display, trying `ddcutil`");
        monitors = ddcutil::enumerate();
        if !monitors.is_empty() {
            log::info!("`ddcutil` found {} display(s)", monitors.len());
        }
    }

    monitors.extend(backlight::enumerate());

    if monitors.is_empty() {
        log::warn!("no display with brightness control was detected by any backend");
    }

    monitors
}

/// Reads the brightness of the display identified by `id`.
///
/// The value is on the display's own scale, which is `0..=255` for DDC/CI
/// displays and `0..=100` for the others. Use [`MonitorInfo::brightness_percent`]
/// for a value that is comparable across displays.
///
/// # Errors
///
/// Returns [`Error::InvalidId`] when `id` is malformed and
/// [`Error::DisplayNotFound`] when no backend knows that display.
pub fn get_brightness(id: &str) -> Result<u8> {
    match MonitorId::parse(id)? {
        parsed if parsed.backend() == Backend::Backlight => backlight::get(parsed.locator()),
        parsed if parsed.backend() == Backend::Ddcutil => ddcutil::get(locator_index(&parsed)?),
        parsed => ddc::get(locator_index(&parsed)?),
    }
}

/// Sets the brightness of the display identified by `id`.
///
/// The value is interpreted on a `0..=100` scale for DDC/CI displays that
/// report a maximum of `100`, and on their own scale otherwise, which matches
/// what the DDC/CI protocol expects.
///
/// # Errors
///
/// Returns [`Error::InvalidId`] when `id` is malformed and
/// [`Error::DisplayNotFound`] when no backend knows that display.
pub fn set_brightness(id: &str, value: u8) -> Result<()> {
    match MonitorId::parse(id)? {
        parsed if parsed.backend() == Backend::Backlight => backlight::set(parsed.locator(), value),
        parsed if parsed.backend() == Backend::Ddcutil => {
            ddcutil::set(locator_index(&parsed)?, value)
        }
        parsed => ddc::set(locator_index(&parsed)?, value),
    }
}

/// Probes the system and explains why a display may not have been detected.
///
/// Never fails: every check degrades to a `false` or an explanatory string.
pub fn diagnose() -> Diagnostics {
    diagnostics::run()
}

/// Extracts the numeric index out of a display identifier.
fn locator_index(id: &MonitorId) -> Result<usize> {
    id.index().ok_or_else(|| Error::InvalidId(id.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_malformed_id_is_rejected_before_any_backend_is_touched() {
        assert!(matches!(
            get_brightness("not-a-display"),
            Err(Error::InvalidId(_))
        ));
        assert!(matches!(
            set_brightness("also-wrong", 50),
            Err(Error::InvalidId(_))
        ));
    }

    #[test]
    fn a_ddc_id_without_an_index_is_rejected() {
        assert!(matches!(
            get_brightness("ddc-abc"),
            Err(Error::InvalidId(_))
        ));
    }

    #[test]
    fn listing_displays_never_fails() {
        // The result depends on the machine, but the call must not panic or
        // return an error on a host without any controllable display.
        let monitors = list_monitors();
        for monitor in monitors {
            assert!(!monitor.id.is_empty());
            assert!(monitor.max >= 1);
            assert!(monitor.brightness <= monitor.max);
        }
    }

    #[test]
    fn diagnostics_never_fail() {
        let diagnostics = diagnose();
        let _ = diagnostics.suggestions.len();
    }
}
