# tauri-brightness-core

Cross-platform display brightness control, with no dependency on any GUI
toolkit.

This crate is the engine behind
[`tauri-plugin-brightness`](https://crates.io/crates/tauri-plugin-brightness).
It is published separately so that a Rust program, a service or a different
plugin can control displays without pulling in Tauri.

## Backends

Three backends are supported, and they are selected automatically:

| Backend | Mechanism | Used for |
| --- | --- | --- |
| DDC/CI over I2C | The pure Rust [`ddc-hi`](https://crates.io/crates/ddc-hi) crate, in process | External monitors on every desktop platform |
| `ddcutil` | The standard Linux DDC/CI command line tool | NVIDIA proprietary drivers, USB-C docks, DisplayLink |
| Kernel backlight | `/sys/class/backlight` on Linux | Internal laptop panels |

The DDC/CI backend is tried first because it avoids spawning a process.
`ddcutil` is only consulted when it found nothing, because it is slower but
covers cases the in-process backend cannot reach. Kernel backlight devices are
always appended, because laptop panels are not DDC/CI devices at all.

## Install

```sh
cargo add tauri-brightness-core
```

## Usage

```rust,no_run
use tauri_brightness_core::{list_monitors, set_brightness};

for monitor in list_monitors() {
    println!("{} is at {}%", monitor.name, monitor.brightness_percent());
    set_brightness(&monitor.id, 50)?;
}

# Ok::<(), tauri_brightness_core::Error>(())
```

Each display is addressed by an identifier that is stable for the lifetime of a
boot, for example `ddc-0`, `ddcutil-1` or `backlight-intel_backlight`.

## Diagnostics

Brightness control depends on permissions that are easy to get wrong, so the
crate can explain what is missing instead of silently doing nothing:

```rust,no_run
let report = tauri_brightness_core::diagnose();
for suggestion in &report.suggestions {
    println!("{suggestion}");
}
```

## Linux permissions

External monitors are reached through `/dev/i2c-*`, which requires the
`i2c-dev` kernel module and usually membership of the `i2c` group:

```sh
sudo modprobe i2c-dev
sudo usermod -aG i2c "$USER"
```

Log out and back in for the group change to take effect. Install `ddcutil` as
well, because it handles the cases the in-process backend cannot:

```sh
sudo apt install ddcutil
```

Internal laptop panels need none of this: they are controlled through the
`org.freedesktop.login1` D-Bus interface, which works without privileges.

## Platform support

| Platform | Supported | Notes |
| --- | --- | --- |
| Linux | ✓ | DDC/CI, `ddcutil` and kernel backlight |
| Windows | Partial | DDC/CI through the Windows API backend of `ddc-hi` |
| macOS | Partial | DDC/CI through the macOS backend of `ddc-hi` |
| Android | ✗ | Not a desktop concept |
| iOS | ✗ | Not a desktop concept |

## Minimum supported Rust version

Rust **1.90**, driven by the `tauri` requirement of the sibling plugin.

## License

Licensed under either of

- Apache License, Version 2.0
- MIT license

at your option.
