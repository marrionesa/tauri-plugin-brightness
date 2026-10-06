//! DDC/CI through the external `ddcutil` command line tool.
//!
//! `ddcutil` is the standard Linux DDC/CI tool and is the only backend that
//! copes with the NVIDIA proprietary driver, USB-C docks and DisplayLink
//! adapters. It is slower than the in-process backend because every operation
//! spawns a process, so it is used as a fallback during detection.
//!
//! Install it with `sudo apt install ddcutil` on Debian and Ubuntu.

use std::process::Command;

use crate::error::command_failed;
use crate::{Backend, Error, MonitorId, MonitorInfo, Result};

/// The `ddcutil` executable name, resolved through `PATH`.
const DDCUTIL: &str = "ddcutil";

/// The VCP feature code for "Brightness".
const BRIGHTNESS_VCP: &str = "10";

/// Enumerates displays by parsing `ddcutil detect`.
///
/// Returns an empty list, rather than an error, when `ddcutil` is missing or
/// reports nothing: detection must stay usable when the helper is unavailable.
pub fn enumerate() -> Vec<MonitorInfo> {
    let Ok(output) = Command::new(DDCUTIL).arg("detect").output() else {
        log::debug!("ddcutil is not installed, skipping this backend");
        return Vec::new();
    };

    if !output.status.success() {
        log::debug!(
            "ddcutil detect failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
        return Vec::new();
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_detect(&stdout)
        .into_iter()
        .map(|display| MonitorInfo {
            id: MonitorId::indexed(Backend::Ddcutil, display.number).to_string(),
            name: display
                .name
                .unwrap_or_else(|| format!("Monitor {}", display.number + 1)),
            brightness: get(display.number).unwrap_or(80),
            max: 100,
            backend: Backend::Ddcutil,
        })
        .collect()
}

/// Reads the brightness of a `ddcutil` display with
/// `ddcutil --display N getvcp 10`.
pub fn get(display_number: usize) -> Result<u8> {
    let output = Command::new(DDCUTIL)
        .args([
            "--display",
            &display_number.to_string(),
            "getvcp",
            BRIGHTNESS_VCP,
        ])
        .output()
        .map_err(|error| Error::Command {
            command: DDCUTIL.to_string(),
            message: error.to_string(),
        })?;

    if !output.status.success() {
        return Err(command_failed(DDCUTIL, &output.stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_getvcp(&stdout).ok_or_else(|| {
        Error::DisplayNotFound(MonitorId::indexed(Backend::Ddcutil, display_number).to_string())
    })
}

/// Writes the brightness of a `ddcutil` display with
/// `ddcutil --display N setvcp 10 <value>`.
pub fn set(display_number: usize, value: u8) -> Result<()> {
    let output = Command::new(DDCUTIL)
        .args([
            "--display",
            &display_number.to_string(),
            "setvcp",
            BRIGHTNESS_VCP,
            &value.to_string(),
        ])
        .output()
        .map_err(|error| Error::Command {
            command: DDCUTIL.to_string(),
            message: error.to_string(),
        })?;

    if output.status.success() {
        Ok(())
    } else {
        Err(command_failed(DDCUTIL, &output.stderr))
    }
}

/// Whether `ddcutil` responds to `--version`.
pub fn available() -> bool {
    Command::new(DDCUTIL)
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// One display as reported by `ddcutil detect`.
#[derive(Debug, PartialEq, Eq)]
pub struct DetectedDisplay {
    /// The display number printed by `ddcutil` and accepted by `--display`.
    ///
    /// `ddcutil` numbers displays from one, so this value is kept exactly as
    /// printed and is used verbatim for both `--display` and the public
    /// identifier.
    pub number: usize,
    /// The model name, when `ddcutil` printed one.
    pub name: Option<String>,
}

/// Parses the display list out of `ddcutil detect` output.
///
/// The relevant shape of the output is:
///
/// ```text
/// Display 1
///    I2C bus:             /dev/i2c-5
///    EDID synopsis:
///       Mfg id:           DEL
///       Model:            DELL U2723QE
///    Monitor:             DEL U2723QE
/// ```
///
/// `ddcutil` numbers displays from one, and that same number is what
/// `--display` expects, so it is preserved unmodified here.
fn parse_detect(stdout: &str) -> Vec<DetectedDisplay> {
    let mut displays: Vec<DetectedDisplay> = Vec::new();
    let mut current: Option<DetectedDisplay> = None;
    let mut from_model_line: Option<String> = None;

    for line in stdout.lines() {
        if let Some(rest) = line.strip_prefix("Display ") {
            if let Some(display) = current.take() {
                displays.push(display);
            }
            from_model_line = None;
            current = rest
                .trim()
                .parse::<usize>()
                .ok()
                .filter(|number| *number > 0)
                .map(|number| DetectedDisplay { number, name: None });
            continue;
        }

        let trimmed = line.trim();

        // `Monitor:` is preferred because it includes the manufacturer.
        if let Some(rest) = trimmed.strip_prefix("Monitor:") {
            let name = rest.trim();
            if !name.is_empty() {
                if let Some(display) = current.as_mut() {
                    display.name = Some(name.to_string());
                }
            }
            continue;
        }

        // `Model:` is the fallback used when the `Monitor:` line is empty.
        if let Some(rest) = trimmed.strip_prefix("Model:") {
            let name = rest.trim();
            if !name.is_empty() {
                from_model_line = Some(name.to_string());
            }
        }
    }

    if let Some(display) = current.take() {
        displays.push(display);
    }

    for display in &mut displays {
        if display.name.is_none() {
            display.name = from_model_line.clone();
        }
    }

    displays
}

/// Parses the current value out of `ddcutil ... getvcp 10` output, which looks
/// like `VCP code 0x10 (Brightness): current value = 88, max value = 100`.
fn parse_getvcp(stdout: &str) -> Option<u8> {
    let after_marker = stdout.split("current value =").nth(1)?;
    let current = after_marker.split(',').next()?.trim();
    current.parse::<u8>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const DETECT_OUTPUT: &str = "\
Display 1
   I2C bus:             /dev/i2c-5
   EDID synopsis:
      Mfg id:           DEL
      Model:            DELL U2723QE
      Product code:     16782
   Monitor:             DEL U2723QE

Display 2
   I2C bus:             /dev/i2c-6
   EDID synopsis:
      Mfg id:           GSM
      Model:            LG HDR 4K
   Monitor:
";

    #[test]
    fn detect_output_is_parsed_keeping_the_ddcutil_numbering() {
        let displays = parse_detect(DETECT_OUTPUT);
        assert_eq!(displays.len(), 2);
        // `ddcutil` numbers displays from one and `--display` needs that same
        // number, so it must not be shifted.
        assert_eq!(displays[0].number, 1);
        assert_eq!(displays[0].name.as_deref(), Some("DEL U2723QE"));
        assert_eq!(displays[1].number, 2);
    }

    #[test]
    fn an_empty_monitor_line_falls_back_to_the_model_line() {
        let displays = parse_detect(DETECT_OUTPUT);
        assert_eq!(displays[1].name.as_deref(), Some("LG HDR 4K"));
    }

    #[test]
    fn malformed_detect_output_yields_no_displays() {
        assert!(parse_detect("").is_empty());
        assert!(parse_detect("no displays found\n").is_empty());
        assert!(parse_detect("Display one\n").is_empty());
        assert!(parse_detect("Display 0\n").is_empty());
    }

    #[test]
    fn getvcp_output_is_parsed() {
        let stdout = "VCP code 0x10 (Brightness): current value = 88, max value = 100";
        assert_eq!(parse_getvcp(stdout), Some(88));
        assert_eq!(parse_getvcp("current value = 0, max value = 100"), Some(0));
        assert_eq!(parse_getvcp("Display not found"), None);
        assert_eq!(parse_getvcp("current value = 300, max value = 100"), None);
    }
}
