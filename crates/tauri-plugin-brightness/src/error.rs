// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Serialize, Serializer};

/// Alias for a [`Result`](std::result::Result) with the error type [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// Errors returned by the plugin commands.
///
/// Every variant serializes to its [`Display`](std::fmt::Display) text, so a
/// JavaScript caller receives a readable string in the rejected promise.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// A brightness backend failed.
    #[error(transparent)]
    Brightness(#[from] tauri_brightness_core::Error),

    /// A Tauri API failed.
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
