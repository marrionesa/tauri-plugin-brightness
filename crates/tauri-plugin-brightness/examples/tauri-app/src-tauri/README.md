# tauri-plugin-brightness-example

The Tauri backend of the tray application example for
[`tauri-plugin-brightness`](../../README.md).

It contains no brightness code: every hardware access goes through the plugin,
which is the point of the example. What lives here is the application shell:

- the plugin registration, in `src/lib.rs`,
- the system tray icon and its menu, in `src/tray.rs`,
- two window helpers the frontend calls, `hide_window` and `quit_app`,
- the capability that grants the plugin commands to the main window.

Author: marrionesa
Project: <https://github.com/marrionesa/tauri-plugin-brightness>
