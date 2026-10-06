//! DDC/CI over I2C, through the pure Rust `ddc-hi` crate.
//!
//! This is the preferred backend because it stays in process. Displays are
//! addressed by their position in the enumeration order, which is stable for
//! the lifetime of a boot.

use ddc_hi::{Ddc, Display};

use crate::{Backend, Error, MonitorId, MonitorInfo, Result};

/// The VCP feature code for "Brightness", defined as `0x10` by the VESA MCCS
/// specification.
pub const BRIGHTNESS_VCP: u8 = 0x10;

/// Enumerates the DDC/CI displays reachable through `ddc-hi`.
///
/// A display that rejects the brightness read is still returned, with the
/// default value, because it is genuinely present and the user should be able
/// to try writing to it.
pub fn enumerate() -> Vec<MonitorInfo> {
    let mut monitors = Vec::new();

    for (index, mut display) in Display::enumerate().into_iter().enumerate() {
        let name = display
            .info
            .model_name
            .as_ref()
            .filter(|name| !name.is_empty())
            .cloned()
            .unwrap_or_else(|| format!("Monitor {}", index + 1));

        let (brightness, max) = match display.handle.get_vcp_feature(BRIGHTNESS_VCP) {
            Ok(value) => {
                let max = value.maximum().clamp(1, 255);
                let brightness = u16::min(value.value(), max);
                (brightness as u8, max as u8)
            }
            Err(error) => {
                log::debug!("could not read brightness of DDC/CI display {index}: {error}");
                (80, 100)
            }
        };

        monitors.push(MonitorInfo {
            id: MonitorId::indexed(Backend::Ddc, index).to_string(),
            name,
            brightness,
            max,
            backend: Backend::Ddc,
        });
    }

    monitors
}

/// Reads the brightness of the DDC/CI display at `index`.
pub fn get(index: usize) -> Result<u8> {
    let mut display = display_at(index)?;
    let value = display
        .handle
        .get_vcp_feature(BRIGHTNESS_VCP)
        .map_err(|error| Error::Ddc(error.to_string()))?;
    let max = value.maximum().clamp(1, 255);
    Ok(u16::min(value.value(), max) as u8)
}

/// Writes the brightness of the DDC/CI display at `index`.
pub fn set(index: usize, value: u8) -> Result<()> {
    let mut display = display_at(index)?;
    display
        .handle
        .set_vcp_feature(BRIGHTNESS_VCP, u16::from(value))
        .map_err(|error| Error::Ddc(error.to_string()))
}

/// Returns the `index`-th enumerated display, or [`Error::DisplayNotFound`].
fn display_at(index: usize) -> Result<Display> {
    Display::enumerate()
        .into_iter()
        .nth(index)
        .ok_or_else(|| Error::DisplayNotFound(MonitorId::indexed(Backend::Ddc, index).to_string()))
}
