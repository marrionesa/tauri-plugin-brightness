# Brightness Control — example application

Tray application that exercises every feature of
[`tauri-plugin-brightness`](../../README.md). It is both the demonstration and
the integration test for the plugin: the app contains no brightness code of its
own, so anything it can do is something the plugin does.

It targets Linux desktop workflows first, with DDC/CI for external monitors and
`/sys/class/backlight` for internal laptop panels.

## Features

- System tray icon with a left-click popup toggle.
- Tray menu with Show, Settings and Quit.
- One slider per detected display.
- Diagnostic panel that appears when no display is detected, explaining which
  permission is missing and printing the command that fixes it.
- Plain HTML/CSS/JS frontend with no framework build step.

## Requirements

- Rust stable 1.90 or newer, the minimum of `tauri` 2.12.
- Bun or Node.js 18+.
- Tauri 2 Linux dependencies.

Debian/Ubuntu packages:

```sh
sudo apt update
sudo apt install -y \
  build-essential \
  curl \
  file \
  libayatana-appindicator3-dev \
  libgtk-3-dev \
  librsvg2-dev \
  libssl-dev \
  libwebkit2gtk-4.1-dev \
  libxdo-dev \
  wget
```

For external monitors on Linux, enable I2C access:

```sh
sudo modprobe i2c-dev
sudo usermod -aG i2c "$USER"
```

Log out and back in after changing groups.

`ddcutil` is optional but recommended:

```sh
sudo apt install ddcutil
```

## Development

```sh
cd crates/tauri-plugin-brightness/examples/tauri-app
bun install
bun run dev
```

The app can also be previewed as static frontend files by opening `src/index.html`;
outside Tauri it uses mock displays.

## Build

```sh
bun run build
```

Tauri writes release artifacts under:

```text
src-tauri/target/release/bundle/
```

## Usage

1. Start the app.
2. Use the tray icon to open the popup.
3. Move a slider to change a display brightness.
4. Open Settings for the author, the version and the DDC/CI information.
5. If no display is detected, the diagnostic panel reports what to fix: the
   `i2c-dev` module, the `i2c` group, or a missing `ddcutil`.

## How it uses the plugin

The application registers the plugin and nothing else:

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_brightness::init())
```

The frontend calls the plugin commands through `window.__TAURI__.core.invoke`,
using their plugin-prefixed names, because this example is built with
`withGlobalTauri`:

```javascript
await invoke("plugin:brightness|list_monitors")
await invoke("plugin:brightness|get_brightness", { id })
await invoke("plugin:brightness|set_brightness", { id, value })
await invoke("plugin:brightness|diagnose")
```

An application with a bundler would instead import the typed bindings:

```typescript
import { listMonitors, setBrightness } from '@tauri-apps/plugin-brightness'
```

The capability grants the plugin commands to the main window:

```json
{ "permissions": ["core:default", "brightness:default"] }
```

## Author

Created and maintained by marrionesa.

- GitHub: <https://github.com/marrionesa>
- Project: <https://github.com/marrionesa/tauri-plugin-brightness>

## License

MIT OR Apache-2.0. See [LICENSE-MIT](./LICENSE-MIT) and
[LICENSE-APACHE](./LICENSE-APACHE).
