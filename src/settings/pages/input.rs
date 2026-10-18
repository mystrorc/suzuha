use amane::Service;

use super::super::button::{self, Style};
use super::super::choice;
use super::super::field;
use super::super::page::Page;
use super::super::row;
use super::super::slider::{self, Range};
use super::super::switch;
use super::super::Settings;

pub const POINTER_DEVICES: [(&str, &str); 3] = [
    ("Logitech G Pro (Mouse)", "gpro"),
    ("Logitech ERGO M575 (Trackball)", "m575"),
    ("All Mice (Global Default)", "global"),
];

pub const GPRO_POLLING_RATES: [(&str, &str); 4] = [
    ("1000 Hz (1ms)", "1ms"),
    ("500 Hz (2ms)", "2ms"),
    ("250 Hz (4ms)", "4ms"),
    ("125 Hz (8ms)", "8ms"),
];

pub const M575_POLLING_RATES: [(&str, &str); 1] = [
    ("125 Hz (8ms - Default)", "8ms"),
];

pub const GPRO_DPI_OPTIONS: [(&str, &str); 8] = [
    ("400 DPI", "400"),
    ("800 DPI (Default)", "800"),
    ("1000 DPI", "1000"),
    ("1200 DPI", "1200"),
    ("1600 DPI", "1600"),
    ("2000 DPI", "2000"),
    ("2400 DPI", "2400"),
    ("3200 DPI", "3200"),
];

pub const M575_DPI_OPTIONS: [(&str, &str); 6] = [
    ("400 DPI", "400"),
    ("800 DPI (Default)", "800"),
    ("1000 DPI", "1000"),
    ("1200 DPI", "1200"),
    ("1600 DPI", "1600"),
    ("2000 DPI", "2000"),
];

pub fn build(page: &mut Page) {
    // Keyboard Configuration
    field::add(
        page,
        "input_keyboard_layout",
        "Keyboard layout",
        "Primary XKB layout code (e.g. us, de, fr, gb)",
    );

    slider::add(
        page,
        "input_keyboard_repeat_rate",
        "Repeat rate",
        "Characters per second while holding down a key",
        Range {
            min: 1.0,
            max: 200.0,
            step: 1.0,
            label: slider::count,
        },
    );

    slider::add(
        page,
        "input_keyboard_repeat_delay",
        "Repeat delay",
        "Delay in milliseconds before key repeating begins",
        Range {
            min: 50.0,
            max: 2000.0,
            step: 25.0,
            label: slider::milliseconds,
        },
    );

    page.end_group();

    // Touchpad Settings
    switch::add(
        page,
        "input_touchpad_natural_scroll",
        "Natural scrolling",
        "Invert scroll direction for two-finger touchpad scrolling",
    );

    switch::add(
        page,
        "input_touchpad_tap_to_click",
        "Tap to click",
        "Tap touchpad surface with one finger for left click",
    );

    switch::add(
        page,
        "input_touchpad_dwt",
        "Disable while typing",
        "Temporarily ignore touchpad input while typing on keyboard",
    );

    page.end_group();

    // Mouse & Pointer Settings (Per-Device Configuration)
    choice::add(
        page,
        "input_pointer_device",
        "Target pointing device",
        "Select pointing device to configure its speed, polling rate, and DPI",
        &POINTER_DEVICES,
    );

    let current_dev = Settings::read().staged("input_pointer_device");

    match current_dev {
        "m575" => {
            slider::add(
                page,
                "input_trackball_speed",
                "Pointer speed",
                "Trackball sensitivity and acceleration scaling for Logitech ERGO M575",
                Range {
                    min: -1.0,
                    max: 1.0,
                    step: 0.05,
                    label: slider::plain,
                },
            );

            switch::add(
                page,
                "input_trackball_flat",
                "Flat acceleration",
                "Disable acceleration curve for 1:1 trackball motion",
            );

            choice::add(
                page,
                "input_trackball_polling_rate",
                "Polling rate (Report rate)",
                "Standard report frequency for Logitech ERGO M575",
                &M575_POLLING_RATES,
            );

            choice::add(
                page,
                "input_trackball_dpi",
                "Hardware sensor DPI",
                "Optical sensor resolution configured via Solaar",
                &M575_DPI_OPTIONS,
            );

            switch::add(
                page,
                "input_trackball_left_handed",
                "Left-handed mode",
                "Swap trackball buttons for left-handed use",
            );
        }
        "global" => {
            slider::add(
                page,
                "input_mouse_sensitivity",
                "Default pointer speed",
                "Global speed and acceleration scaling for connected mice",
                Range {
                    min: -1.0,
                    max: 1.0,
                    step: 0.05,
                    label: slider::plain,
                },
            );

            switch::add(
                page,
                "input_mouse_flat",
                "Flat acceleration",
                "Disable acceleration curve globally for standard mice",
            );

            switch::add(
                page,
                "input_mouse_left_handed",
                "Left-handed mode",
                "Swap primary left and right mouse buttons globally",
            );
        }
        _ => {
            // Default: Logitech G Pro
            slider::add(
                page,
                "input_gpro_speed",
                "Pointer speed",
                "Cursor speed and acceleration scaling for Logitech G Pro",
                Range {
                    min: -1.0,
                    max: 1.0,
                    step: 0.05,
                    label: slider::plain,
                },
            );

            switch::add(
                page,
                "input_gpro_flat",
                "Flat acceleration",
                "Disable acceleration curve for consistent 1:1 motion",
            );

            choice::add(
                page,
                "input_gpro_polling_rate",
                "Polling rate (Report rate)",
                "Sensor report frequency (Hz) configured via Solaar",
                &GPRO_POLLING_RATES,
            );

            choice::add(
                page,
                "input_gpro_dpi",
                "Hardware sensor DPI",
                "Optical sensor resolution configured via Solaar",
                &GPRO_DPI_OPTIONS,
            );

            switch::add(
                page,
                "input_gpro_left_handed",
                "Left-handed mode",
                "Swap primary left and right mouse buttons",
            );
        }
    }

    page.end_group();

    // Action button
    action(
        page,
        "Apply input settings",
        "Write configuration to Niri and reload input devices immediately",
        "Apply",
        Style::Primary,
        Settings::apply,
    );

    page.end_group();
}

fn action(
    page: &mut Page,
    title: &str,
    detail: &str,
    label: &str,
    style: Style,
    on_click: impl Fn() + 'static,
) {
    let control = button::view(page.theme, label, style, true, on_click);

    let row = row::view(
        page.theme,
        page.width,
        row::HEIGHT,
        title,
        detail,
        control,
        button::width(label),
    );

    page.row(row::HEIGHT, row);
}
