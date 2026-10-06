//! Brightness Control — example application for `tauri-plugin-brightness`.
//!
//! The application owns no brightness code at all: every hardware access goes
//! through the plugin, which is the point of the example. It wires up three
//! things:
//!
//! 1. The plugin itself, plus an `opener` plugin for the About link.
//! 2. The system tray icon and its menu (see [`tray`]).
//! 3. Two small window helpers, [`hide_window`] and [`quit_app`], that the
//!    frontend calls to behave like a tray popup.

use tauri::WindowEvent;
use tauri_plugin_brightness::BrightnessExt;

pub mod tray;

/// Hides the main window, called by the frontend when the user clicks away or
/// presses `Escape`.
#[tauri::command]
fn hide_window(window: tauri::Window) -> Result<(), String> {
    window.hide().map_err(|error| error.to_string())
}

/// Quits the application, called from the tray menu and the Settings view.
#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

/// Bootstraps and runs the application. This is the single entry point called
/// from `main.rs`.
pub fn run() {
    // Initialise logging, with `RUST_LOG=info` as the default filter.
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .try_init();

    tauri::Builder::default()
        .plugin(tauri_plugin_brightness::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            tray::setup_tray(app.handle())?;

            // Log what the plugin can see, which is the first thing to check
            // when a slider appears to do nothing.
            let monitors = app.brightness().list_monitors();
            log::info!("{} display(s) with brightness control", monitors.len());
            for monitor in &monitors {
                log::info!(
                    "  {} [{}] {}%",
                    monitor.name,
                    monitor.id,
                    monitor.brightness_percent()
                );
            }

            Ok(())
        })
        .on_tray_icon_event(tray::handle_tray_icon_event)
        .on_menu_event(tray::handle_menu_event)
        .on_window_event(|window, event| {
            // Intercept the close request: hide the window instead of
            // destroying it, so re-opening from the tray stays instant.
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![hide_window, quit_app])
        .run(tauri::generate_context!())
        .expect("error while running the Tauri application");
}
