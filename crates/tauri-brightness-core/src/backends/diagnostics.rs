//! Probing the system to explain why no display was detected.
//!
//! An application that only shows a slider leaves the user guessing when the
//! slider does nothing. This module gathers the environment facts that decide
//! whether DDC/CI can work and turns them into actionable suggestions.

use std::fs;
use std::path::Path;
use std::process::Command;

use crate::backends::{backlight, ddc, ddcutil};
use crate::Diagnostics;

/// Runs every check and returns the collected diagnostics.
pub fn run() -> Diagnostics {
    let i2c_dev_sysfs = Path::new("/sys/class/i2c-dev").exists();
    let i2c_dev_loaded = i2c_dev_is_available(i2c_dev_sysfs);
    let i2c_devices = i2c_device_nodes();
    let (in_i2c_group, in_video_group) = current_groups();

    let ddcutil_available = ddcutil::available();
    let ddcutil_detect_output = if ddcutil_available {
        match Command::new("ddcutil").arg("detect").output() {
            Ok(output) => format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ),
            Err(error) => format!("could not run `ddcutil detect`: {error}"),
        }
    } else {
        "ddcutil is not installed".to_string()
    };

    let ddc_hi_count = ddc::enumerate().len();
    let ddcutil_count = if ddcutil_available {
        ddcutil::enumerate().len()
    } else {
        0
    };
    let backlight_count = backlight::devices().len();
    let has_nvidia = Path::new("/proc/driver/nvidia").exists() || nvidia_in_drm();

    let suggestions = suggestions(
        i2c_dev_loaded,
        i2c_devices.is_empty(),
        in_i2c_group,
        ddcutil_available,
        ddc_hi_count,
        ddcutil_count,
        has_nvidia,
    );

    Diagnostics {
        i2c_dev_loaded,
        i2c_dev_sysfs,
        i2c_devices,
        in_i2c_group,
        in_video_group,
        ddcutil_available,
        ddcutil_detect_output,
        ddc_hi_count,
        ddcutil_count,
        backlight_count,
        has_nvidia,
        suggestions,
    }
}

/// Builds the list of suggestions for the collected environment facts.
///
/// Split out from [`run`] so the wording can be unit tested without touching
/// the real system.
#[allow(clippy::too_many_arguments)]
fn suggestions(
    i2c_dev_loaded: bool,
    no_i2c_devices: bool,
    in_i2c_group: bool,
    ddcutil_available: bool,
    ddc_hi_count: usize,
    ddcutil_count: usize,
    has_nvidia: bool,
) -> Vec<String> {
    let mut suggestions = Vec::new();

    if !i2c_dev_loaded {
        suggestions.push(
            "The `i2c-dev` kernel module is not loaded. Run: sudo modprobe i2c-dev".to_string(),
        );
    }
    if no_i2c_devices {
        suggestions.push(
            "No `/dev/i2c-*` device nodes were found. Run: sudo modprobe i2c-dev (or reboot)"
                .to_string(),
        );
    }
    if !in_i2c_group && !no_i2c_devices {
        suggestions.push(
            "Your user is not in the `i2c` group. Run: sudo usermod -aG i2c $USER, then log out and back in"
                .to_string(),
        );
    }
    if !ddcutil_available {
        suggestions.push(
            "`ddcutil` is not installed. It is required for NVIDIA, docks and USB-DDC monitors. Run: sudo apt install ddcutil"
                .to_string(),
        );
    }
    if has_nvidia && ddcutil_count == 0 {
        suggestions.push(
            "An NVIDIA device was found. Load the I2C module and log out and back in: sudo modprobe i2c_dev"
                .to_string(),
        );
    }
    if ddc_hi_count == 0 && ddcutil_count == 0 && suggestions.is_empty() {
        suggestions.push(
            "Everything looks correctly configured but no display was detected. Check that your monitors have DDC/CI enabled in their on-screen menu"
                .to_string(),
        );
        suggestions.push(
            "Run `ddcutil detect` in a terminal. If it lists displays, the plugin will work too"
                .to_string(),
        );
    }

    suggestions
}

/// Whether the `i2c-dev` support is available to userspace.
///
/// Three cases produce it, and checking only the first would be wrong on a
/// kernel that has `i2c-dev` built in rather than as a loadable module: there is
/// then no `/sys/module/i2c_dev` directory and no `/proc/modules` entry, but the
/// functionality is present, which the `/sys/class/i2c-dev` class proves.
fn i2c_dev_is_available(class_exists: bool) -> bool {
    class_exists || Path::new("/sys/module/i2c_dev").exists() || module_is_loaded("i2c_dev")
}

/// Whether a kernel module is currently loaded, by reading `/proc/modules`.
fn module_is_loaded(name: &str) -> bool {
    let Ok(modules) = fs::read_to_string("/proc/modules") else {
        return false;
    };
    modules
        .lines()
        .any(|line| line.split_whitespace().next() == Some(name))
}

/// The `/dev/i2c-*` nodes that exist.
fn i2c_device_nodes() -> Vec<String> {
    let Ok(entries) = fs::read_dir("/dev") else {
        return Vec::new();
    };

    let mut nodes: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            name.starts_with("i2c-").then(|| format!("/dev/{name}"))
        })
        .collect();
    nodes.sort();
    nodes
}

/// Whether the current user is in the `i2c` and `video` groups.
///
/// The supplementary group ids are read from `/proc/self/status` and mapped
/// back to names through `/etc/group`, which avoids spawning `id` and works the
/// same on every Linux desktop.
fn current_groups() -> (bool, bool) {
    let Ok(status) = fs::read_to_string("/proc/self/status") else {
        return (false, false);
    };

    let Some(groups_line) = status.lines().find(|line| line.starts_with("Groups:")) else {
        return (false, false);
    };

    let ids: Vec<&str> = groups_line
        .trim_start_matches("Groups:")
        .split_whitespace()
        .collect();

    let Ok(entries) = fs::read_to_string("/etc/group") else {
        return (false, false);
    };

    let has = |wanted: &str| -> bool {
        entries.lines().any(|line| {
            let mut fields = line.split(':');
            let name = fields.next().unwrap_or_default();
            let _password = fields.next();
            let gid = fields.next().unwrap_or_default();
            name == wanted && ids.contains(&gid)
        })
    };

    (has("i2c"), has("video"))
}

/// Whether an NVIDIA device is exposed through DRM.
fn nvidia_in_drm() -> bool {
    let Ok(entries) = fs::read_dir("/sys/class/drm") else {
        return false;
    };
    entries
        .filter_map(|entry| entry.ok())
        .any(|entry| entry.file_name().to_string_lossy().contains("nvidia"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_healthy_system_gets_no_instructions() {
        let suggestions = suggestions(true, false, true, true, 2, 0, false);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn missing_i2c_module_is_reported_with_the_fix() {
        let suggestions = suggestions(false, false, true, true, 0, 0, false);
        assert_eq!(suggestions.len(), 1);
        assert!(suggestions[0].contains("sudo modprobe i2c-dev"));
    }

    #[test]
    fn missing_ddcutil_is_reported_with_the_install_command() {
        let suggestions = suggestions(true, false, true, false, 1, 0, false);
        assert!(suggestions
            .iter()
            .any(|item| item.contains("sudo apt install ddcutil")));
    }

    #[test]
    fn nvidia_without_ddcutil_displays_gets_its_own_hint() {
        let suggestions = suggestions(true, false, true, true, 0, 0, true);
        assert!(suggestions.iter().any(|item| item.contains("NVIDIA")));
    }

    #[test]
    fn a_perfectly_configured_system_without_displays_still_gets_advice() {
        let suggestions = suggestions(true, false, true, true, 0, 0, false);
        assert_eq!(suggestions.len(), 2);
        assert!(suggestions
            .iter()
            .any(|item| item.contains("ddcutil detect")));
    }

    #[test]
    fn a_builtin_i2c_dev_is_not_reported_as_missing() {
        // The `/sys/class/i2c-dev` class exists whether the module is loadable
        // or built into the kernel, so it is the reliable signal.
        assert!(i2c_dev_is_available(true));
    }
}
