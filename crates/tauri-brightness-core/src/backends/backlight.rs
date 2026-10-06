//! Linux kernel backlight control through `/sys/class/backlight`.
//!
//! Internal laptop panels are not DDC/CI devices, so this backend is always
//! appended to the detected displays and never replaces the DDC/CI ones.
//!
//! Writing a raw value into `/sys` normally requires either `root` or
//! membership of the `video` group, which a desktop application must not
//! demand. Three write paths are therefore tried in order of decreasing
//! privilege requirement:
//!
//! 1. The `org.freedesktop.login1.Session.SetBrightness` D-Bus method, which is
//!    what GNOME, KDE and COSMIC use, and which works through polkit without
//!    `root` and without being in any group.
//! 2. The `brightnessctl` helper, if it happens to be installed.
//! 3. A direct write to the sysfs file, which works when the user is in the
//!    `video` group or is `root`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::{raw_from_percent, Backend, Error, MonitorId, MonitorInfo, Result};

/// The sysfs class holding every backlight device.
const BACKLIGHT_CLASS: &str = "/sys/class/backlight";

/// Enumerates the kernel backlight devices.
///
/// Returns an empty list on non-Linux targets and when the directory is
/// missing, so callers do not need to branch on the operating system.
pub fn enumerate() -> Vec<MonitorInfo> {
    let mut monitors = Vec::new();

    for device in devices() {
        let Some((current, max)) = read_raw(&device) else {
            continue;
        };

        let name = device_name(&device).unwrap_or_else(|| device.clone());

        monitors.push(MonitorInfo {
            id: MonitorId::new(Backend::Backlight, device.clone()).to_string(),
            name: format!("{name} (internal panel)"),
            brightness: crate::percent(current as u8, max as u8),
            max: 100,
            backend: Backend::Backlight,
        });
    }

    monitors
}

/// Reads the brightness of a backlight device as a percentage.
pub fn get(device: &str) -> Result<u8> {
    let (current, max) = read_raw(device).ok_or_else(|| not_found(device))?;
    if max == 0 {
        return Err(Error::Os(format!(
            "`{device}` reports a maximum brightness of zero"
        )));
    }
    Ok(crate::percent(current as u8, max as u8))
}

/// Writes the brightness of a backlight device as a percentage.
pub fn set(device: &str, percent: u8) -> Result<()> {
    let (_current, max) = read_raw(device).ok_or_else(|| not_found(device))?;
    let raw = raw_from_percent(percent, max as u8);

    match set_via_logind(device, raw) {
        Ok(()) => return Ok(()),
        Err(error) => log::debug!("login1 SetBrightness failed for `{device}`: {error}"),
    }

    match set_via_brightnessctl(device, percent) {
        Ok(()) => return Ok(()),
        Err(error) => log::debug!("brightnessctl failed for `{device}`: {error}"),
    }

    let path = PathBuf::from(BACKLIGHT_CLASS)
        .join(device)
        .join("brightness");
    fs::write(&path, raw.to_string()).map_err(|error| {
        Error::Os(format!(
            "could not change the brightness of `{device}`: {error}. \
             Add your user to the `video` group, or install `brightnessctl`."
        ))
    })
}

/// The names of the available backlight devices.
pub fn devices() -> Vec<String> {
    let Ok(entries) = fs::read_dir(BACKLIGHT_CLASS) else {
        return Vec::new();
    };

    let mut devices: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    devices.sort();
    devices
}

/// Reads the `brightness` and `max_brightness` files of a device.
fn read_raw(device: &str) -> Option<(u32, u32)> {
    let base = Path::new(BACKLIGHT_CLASS).join(device);
    let max = read_number(&base.join("max_brightness"))?;
    let current = read_number(&base.join("brightness"))?;
    Some((current, max))
}

/// Reads a sysfs file holding a single unsigned integer.
fn read_number(path: &Path) -> Option<u32> {
    fs::read_to_string(path).ok()?.trim().parse().ok()
}

/// The `type` file of a device, for example `raw` or `platform`, used to make
/// the display name a little more informative than the raw device name.
fn device_name(device: &str) -> Option<String> {
    let path = Path::new(BACKLIGHT_CLASS).join(device).join("type");
    let kind = fs::read_to_string(path).ok()?.trim().to_string();
    if kind.is_empty() {
        None
    } else {
        Some(kind)
    }
}

/// Builds the error returned when a device disappears between detection and
/// access, which happens on external displays that are unplugged.
fn not_found(device: &str) -> Error {
    Error::DisplayNotFound(MonitorId::new(Backend::Backlight, device).to_string())
}

/// Writes through the `login1` D-Bus method, the same path desktop
/// environments use, which needs no privileges thanks to polkit.
fn set_via_logind(device: &str, raw: u8) -> Result<()> {
    let output = Command::new("dbus-send")
        .args([
            "--system",
            "--dest=org.freedesktop.login1",
            "--type=method_call",
            "/org/freedesktop/login1/session/auto",
            "org.freedesktop.login1.Session.SetBrightness",
            "string:backlight",
            &format!("string:{device}"),
            &format!("uint32:{raw}"),
        ])
        .output()
        .map_err(|error| Error::Command {
            command: "dbus-send".to_string(),
            message: error.to_string(),
        })?;

    if output.status.success() {
        Ok(())
    } else {
        Err(crate::error::command_failed("dbus-send", &output.stderr))
    }
}

/// Writes through the optional `brightnessctl` helper.
fn set_via_brightnessctl(device: &str, percent: u8) -> Result<()> {
    let output = Command::new("brightnessctl")
        .args(["-d", device, "set", &format!("{percent}%")])
        .output()
        .map_err(|error| Error::Command {
            command: "brightnessctl".to_string(),
            message: error.to_string(),
        })?;

    if output.status.success() {
        Ok(())
    } else {
        Err(crate::error::command_failed(
            "brightnessctl",
            &output.stderr,
        ))
    }
}
