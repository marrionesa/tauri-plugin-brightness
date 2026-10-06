# Changelog

All notable changes to this project are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
the project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Both crates and the npm package are versioned together, even when only one of
them changed, which is the convention used by the official Tauri plugins.

## [Unreleased]

## [0.1.0]

Initial release.

### Added

- `tauri-brightness-core` crate: DDC/CI over I2C through `ddc-hi`, a `ddcutil`
  fallback, Linux kernel backlight support, structured diagnostics and unit
  tests for every parser.
- `tauri-plugin-brightness` crate with the `list_monitors`, `get_brightness`,
  `set_brightness` and `diagnose` commands, Tauri permission definitions and a
  Rust API behind the `BrightnessExt` extension trait.
- `@tauri-apps/plugin-brightness` npm package with typed JavaScript bindings,
  including an IIFE bundle for applications built with `withGlobalTauri`.
- Tray application example in `crates/tauri-plugin-brightness/examples/tauri-app`,
  including the diagnostic panel that explains missing I2C permissions.

[Unreleased]: https://github.com/marrionesa/tauri-plugin-brightness/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/marrionesa/tauri-plugin-brightness/releases/tag/v0.1.0
