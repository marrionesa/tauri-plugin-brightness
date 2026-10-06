//! System-tray setup and event handlers.
//!
//! Tauri 2's tray API lives in [`tauri::tray`] (the equivalent of Tauri 1's
//! `tauri::SystemTray`). This module:
//!
//! 1. Builds the right-click context menu (Show / Settings / --- / Quit).
//! 2. Builds the tray icon itself via [`tauri::tray::TrayIconBuilder`] using
//!    the app's default window icon.
//! 3. Exposes `handle_tray_icon_event` and `handle_menu_event` — these are
//!    registered as global callbacks on the Tauri builder from
//!    [`crate::lib::run`].
//!
//! The tray icon is created *programmatically* here (NOT declaratively via
//! `app.trayIcon` in `tauri.conf.json`) so we can attach menu items and event
//! handlers in a single place. If you prefer the declarative route, remove
//! this module, add an `app.trayIcon` block to `tauri.conf.json`, and wire
//! `on_tray_icon_event` / `on_menu_event` directly in `lib.rs`.

use tauri::{
    image::Image,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, Size, WebviewWindow,
};

const TRAY_ICON_BYTES: &[u8] = include_bytes!("../icons/tray-32x32.png");
const WINDOW_WIDTH: u32 = 360;
const WINDOW_HEIGHT: u32 = 300;
const COLLAPSED_SIZE: u32 = 44;
const TRAY_MARGIN: i32 = 8;

/// Build the tray icon + menu. Called once from [`crate::lib::run`]'s `setup`
/// hook.
pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    // --- Menu -----------------------------------------------------------
    // Each item gets a stable string id so the menu-event handler can match
    // on it. `MenuItem::with_id(app, id, label, enabled, accelerator)`.
    let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    // Build the menu with `Menu::new` + `append` rather than
    // `Menu::with_items(&[&T])` because the slice contains both `MenuItem`
    // and `PredefinedMenuItem` (different concrete `T`s). `append` accepts
    // any `T: IsMenuItem` per call, so heterogeneous items work fine.
    let menu = Menu::new(app)?;
    menu.append(&show)?;
    menu.append(&settings)?;
    menu.append(&separator)?;
    menu.append(&quit)?;

    // --- Tray icon ------------------------------------------------------
    // Use an explicit PNG embedded in the binary. Relying on the default
    // window icon can leave Linux tray implementations with no usable image.
    let icon = Image::from_bytes(TRAY_ICON_BYTES)?;

    let _tray = TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .tooltip("Brightness Control")
        .menu(&menu)
        // Don't pop the menu on left-click — left-click toggles the popup.
        .show_menu_on_left_click(false)
        // On macOS, render the icon as a "template" image so it adapts to
        // light/dark mode automatically.
        .icon_as_template(cfg!(target_os = "macos"))
        .build(app)?;

    log::info!("system tray initialized");
    Ok(())
}

/// Global tray-icon event handler. Registered on the Tauri builder via
/// `Builder::on_tray_icon_event`.
///
/// Behaviour: a single left-click toggles the main window — hide if visible,
/// otherwise position near the tray area and show.
pub fn handle_tray_icon_event(app: &AppHandle, event: TrayIconEvent) {
    if let TrayIconEvent::Click {
        position,
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        toggle_main_window(app, Some((position.x as i32, position.y as i32)));
    }
}

/// Global menu-event handler. Registered on the Tauri builder via
/// `Builder::on_menu_event`.
pub fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id().as_ref() {
        "show" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = reveal_from_tray(&window, None);
                let _ = window.set_focus();
            }
        }
        "settings" => {
            if let Some(window) = app.get_webview_window("main") {
                // Make sure the panel is visible before emitting.
                let _ = reveal_from_tray(&window, None);
                let _ = window.set_focus();
                // Tell the frontend to swap to the settings view.
                let _ = window.emit("open-settings", ());
            }
        }
        "quit" => {
            log::info!("quit requested from tray menu");
            app.exit(0);
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Toggle the main panel: hide if visible, otherwise expand it from the tray
/// corner so it feels attached to the system tray instead of appearing as a
/// detached popup.
fn toggle_main_window(app: &AppHandle, tray_position: Option<(i32, i32)>) {
    let Some(window) = app.get_webview_window("main") else {
        log::warn!("main window not found");
        return;
    };

    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
    } else {
        let _ = reveal_from_tray(&window, tray_position);
        let _ = window.set_focus();
    }
}

/// Expand the hidden window from the tray corner to the final panel size.
fn reveal_from_tray(
    window: &WebviewWindow,
    tray_position: Option<(i32, i32)>,
) -> tauri::Result<()> {
    window.set_size(Size::Physical(PhysicalSize::new(
        COLLAPSED_SIZE,
        COLLAPSED_SIZE,
    )))?;
    position_dropdown(window, COLLAPSED_SIZE, COLLAPSED_SIZE, tray_position)?;
    window.show()?;

    let frames = [
        (96, 64),
        (180, 120),
        (270, 210),
        (WINDOW_WIDTH, WINDOW_HEIGHT),
    ];

    for (width, height) in frames {
        window.set_size(Size::Physical(PhysicalSize::new(width, height)))?;
        position_dropdown(window, width, height, tray_position)?;
        std::thread::sleep(std::time::Duration::from_millis(14));
    }

    Ok(())
}

fn position_dropdown(
    window: &WebviewWindow,
    width: u32,
    height: u32,
    tray_position: Option<(i32, i32)>,
) -> tauri::Result<()> {
    if let Some(monitor) = window.current_monitor()?.or(window.primary_monitor()?) {
        let mon_size = monitor.size();
        let mon_pos = monitor.position();
        let mon_w = mon_size.width as i32;
        let mon_h = mon_size.height as i32;
        let win_w = width as i32;
        let win_h = height as i32;

        let (anchor_x, anchor_y) =
            tray_position.unwrap_or((mon_pos.x + mon_w - TRAY_MARGIN, mon_pos.y + TRAY_MARGIN));
        let mut x = anchor_x - win_w / 2;
        let mut y = if anchor_y < mon_pos.y + mon_h / 2 {
            anchor_y + TRAY_MARGIN
        } else {
            anchor_y - win_h - TRAY_MARGIN
        };

        x = x.clamp(
            mon_pos.x + TRAY_MARGIN,
            mon_pos.x + mon_w - win_w - TRAY_MARGIN,
        );
        y = y.clamp(
            mon_pos.y + TRAY_MARGIN,
            mon_pos.y + mon_h - win_h - TRAY_MARGIN,
        );

        window.set_position(PhysicalPosition::new(x, y))?;
    }

    Ok(())
}
