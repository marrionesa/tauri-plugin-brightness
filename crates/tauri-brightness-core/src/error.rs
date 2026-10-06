//! Error type shared by every brightness backend.

/// Alias for a [`Result`](std::result::Result) with the error type [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// Errors returned by the brightness backends.
///
/// The variant is chosen so callers can react programmatically (for example a
/// UI that re-runs detection on [`Error::NoDisplays`]) while still showing the
/// [`Display`](std::fmt::Display) implementation to the user.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// The identifier string could not be parsed into a [`MonitorId`].
    ///
    /// [`MonitorId`]: crate::MonitorId
    #[error("invalid display id `{0}`")]
    InvalidId(String),

    /// The identifier is well formed but no backend knows that display.
    #[error("display `{0}` was not found")]
    DisplayNotFound(String),

    /// No display with brightness control was detected by any enabled backend.
    #[error("no display with brightness control was detected")]
    NoDisplays,

    /// The caller asked for a brightness value above the display's maximum.
    #[error("brightness {requested} is above the maximum of {max} for display `{id}`")]
    OutOfRange {
        /// The display that rejected the value.
        id: String,
        /// The requested value.
        requested: u8,
        /// The display's maximum value.
        max: u8,
    },

    /// An external helper process (`ddcutil`, `dbus-send`, `brightnessctl`)
    /// exited with a non-zero status.
    #[error("`{command}` failed: {message}")]
    Command {
        /// The helper that failed.
        command: String,
        /// Its stderr, trimmed.
        message: String,
    },

    /// The display rejects DDC/CI writes, usually because of missing
    /// permissions on the I2C device node.
    #[error("display `{0}` rejected the DDC/CI request")]
    DdcRejected(String),

    /// An operating system level failure such as a missing sysfs entry.
    #[error("{0}")]
    Os(String),

    /// Low-level DDC/CI transport error.
    ///
    /// `ddc-hi` reports transport failures as `anyhow::Error`, which does not
    /// implement [`std::error::Error`], so the rendered message is kept here.
    #[error("DDC/CI transport error: {0}")]
    Ddc(String),
}

impl Error {
    /// Returns `true` when re-running detection may produce a different result,
    /// for example after the user fixes I2C permissions.
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::NoDisplays | Self::DdcRejected(_))
    }
}

/// Builds an [`Error::Command`] from a failed helper invocation.
pub(crate) fn command_failed(command: &str, stderr: &[u8]) -> Error {
    let message = String::from_utf8_lossy(stderr).trim().to_string();
    Error::Command {
        command: command.to_string(),
        message: if message.is_empty() {
            "exited with a non-zero status".to_string()
        } else {
            message
        },
    }
}
