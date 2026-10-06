//! Prints what the crate detects on this machine, and the fixes it suggests
//! when nothing was found.
//!
//! Run it with `cargo run --example probe` to check a new machine before
//! opening an issue:
//!
//! ```text
//! detected 1 display(s)
//!   id=ddc-0  name=S24B150  100/100  backend=Ddc
//! i2c_dev_loaded=true  in_i2c_group=true  ddcutil=true  nvidia=true
//! ```

fn main() {
    let monitors = tauri_brightness_core::list_monitors();

    println!("detected {} display(s)", monitors.len());
    for monitor in &monitors {
        println!(
            "  id={}  name={}  {}/{}  backend={:?}",
            monitor.id, monitor.name, monitor.brightness, monitor.max, monitor.backend
        );
    }

    let report = tauri_brightness_core::diagnose();
    println!(
        "i2c_dev_loaded={}  in_i2c_group={}  ddcutil={}  nvidia={}",
        report.i2c_dev_loaded, report.in_i2c_group, report.ddcutil_available, report.has_nvidia
    );
    println!(
        "displays: ddc-hi={}  ddcutil={}  backlight={}",
        report.ddc_hi_count, report.ddcutil_count, report.backlight_count
    );

    for suggestion in &report.suggestions {
        println!("  suggestion: {suggestion}");
    }
}
