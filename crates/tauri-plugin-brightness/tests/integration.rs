//! Integration test that exercises the plugin against the real machine.
//!
//! It does not open a window and it does not need a display server, so it runs
//! on CI as well as on a desktop. The assertions are deliberately about
//! consistency rather than about specific hardware values, because the result
//! depends on the machine.
//!
//! `set_brightness` is not exercised here: changing a real display from a test
//! would be a side effect on the user's screen. The example application is the
//! place for that.

use tauri_brightness_core::{get_brightness, list_monitors, percent, Backend, Error};

#[test]
fn detected_displays_report_a_consistent_state() {
    for monitor in list_monitors() {
        assert!(
            !monitor.id.is_empty(),
            "a display must always carry an identifier"
        );
        assert!(
            monitor.max >= 1,
            "the maximum of {} must be at least 1",
            monitor.id
        );
        assert!(
            monitor.brightness <= monitor.max,
            "the brightness of {} is above its maximum",
            monitor.id
        );
        assert!(
            monitor.brightness_percent() <= 100,
            "the percentage of {} is above 100",
            monitor.id
        );
        assert!(
            !monitor.name.is_empty(),
            "a display must always carry a name"
        );
    }
}

#[test]
fn every_identifier_round_trips_and_dispatches_to_the_right_backend() {
    for monitor in list_monitors() {
        // A second read through the public API must either agree with the
        // enumerated value or report a transport error. Enumeration is
        // deliberately forgiving: a display that is present but rejects the
        // brightness feature is still listed, so that the user can try writing
        // to it, which is why a re-read may legitimately fail here.
        match get_brightness(&monitor.id) {
            Ok(reread) => assert!(
                reread <= monitor.max,
                "{} reported {reread}, above its maximum of {}",
                monitor.id,
                monitor.max
            ),
            Err(Error::Ddc(_)) => assert!(
                monitor.backend == Backend::Ddc,
                "only a DDC/CI display may report a DDC transport error"
            ),
            Err(error) => panic!("could not re-read {}: {error}", monitor.id),
        }

        let prefix = monitor.id.split_once('-').map(|(prefix, _)| prefix);
        let expected = match monitor.backend {
            Backend::Ddc => "ddc",
            Backend::Ddcutil => "ddcutil",
            Backend::Backlight => "backlight",
        };
        assert_eq!(
            prefix,
            Some(expected),
            "the identifier `{}` must name its own backend",
            monitor.id
        );
    }
}

#[test]
fn an_unknown_display_id_is_rejected_instead_of_panicking() {
    let error = get_brightness("ddc-9999").unwrap_err();
    assert!(
        matches!(error, Error::DisplayNotFound(_) | Error::Ddc(_)),
        "unexpected error for a missing display: {error}"
    );

    assert!(matches!(
        get_brightness("nonsense"),
        Err(Error::InvalidId(_))
    ));
}

#[test]
fn diagnostics_describe_the_machine_without_failing() {
    let report = tauri_brightness_core::diagnose();

    // The DDC/CI count must agree with what enumeration reports, because both
    // come from the same backend.
    let enumerated = list_monitors()
        .iter()
        .filter(|monitor| monitor.backend == Backend::Ddc)
        .count();
    assert_eq!(report.ddc_hi_count, enumerated);

    // The sysfs flag and the device list must not contradict each other.
    assert!(
        report.i2c_dev_sysfs || report.i2c_devices.is_empty(),
        "device nodes were listed while the sysfs class is missing"
    );

    // When nothing was detected, the report must tell the user what to do.
    let total = report.ddc_hi_count + report.ddcutil_count + report.backlight_count;
    if total == 0 {
        assert!(
            !report.suggestions.is_empty(),
            "a report with no display must always suggest a fix"
        );
    }
}

#[test]
fn percent_conversion_is_the_contract_the_ui_relies_on() {
    assert_eq!(percent(80, 100), 80);
    assert_eq!(percent(255, 255), 100);
    assert_eq!(percent(0, 0), 0);
}
