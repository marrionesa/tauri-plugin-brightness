//! Brightness backends, tried in a fixed order.
//!
//! Each module implements one mechanism and exposes the same three operations:
//! enumerate, read and write. The public API in the crate root picks between
//! them from the [`Backend`](crate::Backend) encoded in a display identifier.
//!
//! Detection order, and why it matters:
//!
//! 1. [`ddc`] talks DDC/CI over I2C in-process and is far faster than spawning
//!    a helper, but it cannot drive monitors behind the NVIDIA proprietary
//!    driver, USB-C docks or DisplayLink adapters.
//! 2. [`ddcutil`] shells out to the standard Linux DDC/CI tool, which does
//!    handle those cases, and is only consulted when the first backend found
//!    nothing.
//! 3. [`backlight`] covers internal laptop panels, which are not DDC/CI devices
//!    at all and are therefore always appended to the result.

pub mod backlight;
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
pub mod ddc;
pub mod ddcutil;
pub mod diagnostics;
