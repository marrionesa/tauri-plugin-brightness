//! Shared data types: display identifiers, monitor descriptions and
//! diagnostics.

use serde::{Deserialize, Serialize};

/// Which mechanism a display is driven through.
///
/// The backend is encoded in the textual form of a [`MonitorId`], so a display
/// can be addressed again in a later call without keeping state.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    /// DDC/CI over I2C through the pure Rust `ddc-hi` crate.
    Ddc,
    /// DDC/CI through the external `ddcutil` command line tool.
    Ddcutil,
    /// Linux kernel backlight interface at `/sys/class/backlight`.
    Backlight,
}

impl Backend {
    /// The lowercase prefix used in the textual identifier form.
    pub fn prefix(self) -> &'static str {
        match self {
            Self::Ddc => "ddc",
            Self::Ddcutil => "ddcutil",
            Self::Backlight => "backlight",
        }
    }
}

/// A display identifier that can be round-tripped through its textual form.
///
/// The textual form is `"{backend}-{index}"` for DDC/CI displays and
/// `"{backend}-{name}"` for kernel backlight devices, for example `ddc-0`,
/// `ddcutil-1` or `backlight-intel_backlight`. It is stable for the lifetime of
/// a boot and is what the command layer accepts and returns.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(try_from = "String", into = "String")]
pub struct MonitorId {
    backend: Backend,
    /// Numeric index for DDC/CI backends, sysfs device name for backlight.
    locator: String,
}

impl MonitorId {
    /// Creates an identifier from a backend and a locator.
    ///
    /// The locator is used verbatim, so callers should pass the device name or
    /// index without any prefix.
    pub fn new(backend: Backend, locator: impl Into<String>) -> Self {
        Self {
            backend,
            locator: locator.into(),
        }
    }

    /// Creates an identifier addressed by a display index, for the DDC/CI
    /// backends.
    pub fn indexed(backend: Backend, index: usize) -> Self {
        Self::new(backend, index.to_string())
    }

    /// The backend this identifier refers to.
    pub fn backend(&self) -> Backend {
        self.backend
    }

    /// The backend specific locator.
    pub fn locator(&self) -> &str {
        &self.locator
    }

    /// The display index, for the DDC/CI backends.
    pub fn index(&self) -> Option<usize> {
        self.locator.parse().ok()
    }

    /// Parses the textual form produced by [`MonitorId::to_string`].
    ///
    /// Any textual form is accepted, including one that uses the same prefix as
    /// a different backend, because the split is done on the first hyphen.
    pub fn parse(value: &str) -> crate::Result<Self> {
        let Some((prefix, locator)) = value.split_once('-') else {
            return Err(crate::Error::InvalidId(value.to_string()));
        };
        if locator.is_empty() {
            return Err(crate::Error::InvalidId(value.to_string()));
        }
        let backend = match prefix {
            "ddc" => Backend::Ddc,
            "ddcutil" => Backend::Ddcutil,
            "backlight" => Backend::Backlight,
            _ => return Err(crate::Error::InvalidId(value.to_string())),
        };
        Ok(Self::new(backend, locator))
    }
}

impl std::fmt::Display for MonitorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.backend.prefix(), self.locator)
    }
}

impl TryFrom<String> for MonitorId {
    type Error = crate::Error;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<MonitorId> for String {
    fn from(value: MonitorId) -> Self {
        value.to_string()
    }
}

/// A display whose brightness can be read and written.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MonitorInfo {
    /// Identifier to pass back to the read and write functions.
    pub id: String,
    /// Human readable model name, or a generated fallback such as `Monitor 1`.
    pub name: String,
    /// Current brightness, on a `0..=max` scale.
    pub brightness: u8,
    /// Highest value the display accepts.
    pub max: u8,
    /// Which mechanism drives this display.
    pub backend: Backend,
}

impl MonitorInfo {
    /// The current brightness as a percentage of the maximum.
    ///
    /// Returns `0` when the maximum is zero, which cannot normally happen
    /// because the reported maximum is always clamped to at least `1`.
    pub fn brightness_percent(&self) -> u8 {
        percent(self.brightness, self.max)
    }
}

/// Converts a raw brightness value to a percentage of `max`, clamped to
/// `0..=100`.
///
/// This is the single place where the `0..=100` UI scale meets the raw scale
/// reported by the hardware, so it is covered by unit tests.
pub fn percent(value: u8, max: u8) -> u8 {
    if max == 0 {
        return 0;
    }
    let pct = (u32::from(value) * 100 / u32::from(max)).min(100);
    pct as u8
}

/// Converts a percentage into a raw brightness value for a display whose
/// maximum is `max`.
pub fn raw_from_percent(percent: u8, max: u8) -> u8 {
    let raw = (u32::from(percent) * u32::from(max)) / 100;
    raw.min(u32::from(max)) as u8
}

/// Result of probing the system for brightness capabilities.
///
/// The plugin exposes this so an application can explain to the user why no
/// display was detected, instead of showing a control that does nothing.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostics {
    /// Whether the `i2c-dev` kernel module is loaded.
    pub i2c_dev_loaded: bool,
    /// Whether `/sys/class/i2c-dev` exists.
    pub i2c_dev_sysfs: bool,
    /// The `/dev/i2c-*` device nodes that were found.
    pub i2c_devices: Vec<String>,
    /// Whether the current user belongs to the `i2c` group.
    pub in_i2c_group: bool,
    /// Whether the current user belongs to the `video` group.
    pub in_video_group: bool,
    /// Whether `ddcutil` is on `PATH`.
    pub ddcutil_available: bool,
    /// Raw `ddcutil detect` output, useful when filing a bug report.
    pub ddcutil_detect_output: String,
    /// Number of displays the `ddc-hi` backend detected.
    pub ddc_hi_count: usize,
    /// Number of displays the `ddcutil` backend detected.
    pub ddcutil_count: usize,
    /// Number of kernel backlight devices detected.
    pub backlight_count: usize,
    /// Whether an NVIDIA device was found, which usually means the proprietary
    /// driver needs `ddcutil` rather than raw I2C access.
    pub has_nvidia: bool,
    /// Actionable, human readable suggestions for fixing an empty detection.
    pub suggestions: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_maps_the_full_range() {
        assert_eq!(percent(0, 100), 0);
        assert_eq!(percent(50, 100), 50);
        assert_eq!(percent(100, 100), 100);
        assert_eq!(percent(255, 255), 100);
        assert_eq!(percent(128, 255), 50);
    }

    #[test]
    fn percent_is_total_and_never_divides_by_zero() {
        assert_eq!(percent(7, 0), 0);
        assert_eq!(percent(255, 1), 100);
    }

    #[test]
    fn raw_from_percent_is_the_inverse_on_round_numbers() {
        assert_eq!(raw_from_percent(50, 100), 50);
        assert_eq!(raw_from_percent(100, 100), 100);
        assert_eq!(raw_from_percent(0, 255), 0);
        assert_eq!(raw_from_percent(100, 255), 255);
    }

    #[test]
    fn monitor_id_round_trips_through_its_textual_form() {
        for id in [
            MonitorId::indexed(Backend::Ddc, 0),
            MonitorId::new(Backend::Ddcutil, "3"),
            MonitorId::new(Backend::Backlight, "intel_backlight"),
        ] {
            let text = id.to_string();
            assert_eq!(MonitorId::parse(&text).unwrap(), id);
        }
    }

    #[test]
    fn monitor_id_keeps_hyphens_inside_the_locator() {
        let id = MonitorId::parse("backlight-amdgpu_bl0").unwrap();
        assert_eq!(id.backend(), Backend::Backlight);
        assert_eq!(id.locator(), "amdgpu_bl0");
    }

    #[test]
    fn monitor_id_rejects_malformed_input() {
        for bad in ["", "ddc", "ddc-", "unknown-0", "-0"] {
            assert!(
                MonitorId::parse(bad).is_err(),
                "`{bad}` should not parse as a display id"
            );
        }
    }

    #[test]
    fn brightness_percent_uses_the_display_maximum() {
        let monitor = MonitorInfo {
            id: "ddc-0".into(),
            name: "Monitor 1".into(),
            brightness: 128,
            max: 255,
            backend: Backend::Ddc,
        };
        assert_eq!(monitor.brightness_percent(), 50);
    }

    #[test]
    fn monitor_info_serializes_with_camel_case_keys() {
        let monitor = MonitorInfo {
            id: "ddc-0".into(),
            name: "Dell U2723QE".into(),
            brightness: 80,
            max: 100,
            backend: Backend::Ddcutil,
        };
        let json = serde_json::to_string(&monitor).unwrap();
        assert!(json.contains("\"max\":100"));
        assert!(json.contains("\"backend\":\"ddcutil\""));
    }
}
