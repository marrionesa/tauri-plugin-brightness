# Changelog

All notable changes to this project are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
the project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Both crates and the npm package are versioned together, even when only one of
them changed, which is the convention used by the official Tauri plugins.

## [Unreleased]

## [0.1.1] - 2026-10-06

### Fixed

- **Security:** a backlight display identifier such as
  `backlight-../../etc/passwd` was accepted and turned into a path outside
  `/sys/class/backlight`, which would have let a caller read or attempt to write
  an arbitrary file as the user running the application. Identifiers are now
  validated when parsed and again where the path is built. See
  [SECURITY.md](SECURITY.md) for the details and the two tests that cover it.

## [0.1.0]

Initial release.

### Added

- `tauri-brightness-core` crate: DDC/CI over I2C through `ddc-hi`, a `ddcutil`
  fallback, Linux kernel backlight support, structured diagnostics and unit
  tests for every parser.
- `tauri-plugin-brightness` crate with the `list_monitors`, `get_brightness`,
  `set_brightness` and `diagnose` commands, Tauri permission definitions and a
  Rust API behind the `BrightnessExt` extension trait.
- `tauri-plugin-brightness-api` npm package with typed JavaScript bindings,
  including an IIFE bundle for applications built with `withGlobalTauri`.
- Tray application example in `crates/tauri-plugin-brightness/examples/tauri-app`,
  including the diagnostic panel that explains missing I2C permissions.

[Unreleased]: https://github.com/marrionesa/tauri-plugin-brightness/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/marrionesa/tauri-plugin-brightness/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/marrionesa/tauri-plugin-brightness/releases/tag/v0.1.0
