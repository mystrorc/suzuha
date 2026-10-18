use super::super::button::{self, Style};
use super::super::choice;
use super::super::field;
use super::super::page::Page;
use super::super::row;
use super::super::slider::{self, Range};
use super::super::switch;
use super::super::Settings;

pub const DPI_OPTIONS: [(&str, &str); 6] = [
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

    field::add(
        page,
        "input_keyboard_variant",
        "Layout variant",
        "Keyboard variant code (e.g. altgr-intl, dvorak)",
    );

    field::add(
        page,
        "input_keyboard_options",
        "XKB options",
        "XKB rule options (e.g. ctrl:nocaps, compose:ralt)",
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

    // Mouse Settings
    slider::add(
        page,
        "input_mouse_sensitivity",
        "Pointer speed",
        "Speed and acceleration scaling for connected mice",
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
        "Disable acceleration curve for consistent 1:1 motion",
    );

    switch::add(
        page,
        "input_mouse_left_handed",
        "Left-handed mode",
        "Swap primary left and right mouse buttons",
    );

    page.end_group();

    // Logitech Trackball (ERGO M575) Settings
    slider::add(
        page,
        "input_trackball_sensitivity",
        "Trackball speed",
        "Sensitivity and speed for Logitech ERGO M575 trackball",
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
        "Disable acceleration curve for trackball motion",
    );

    switch::add(
        page,
        "input_trackball_left_handed",
        "Left-handed trackball",
        "Swap trackball buttons for left-handed use",
    );

    choice::add(
        page,
        "input_trackball_dpi",
        "Hardware sensor DPI",
        "Optical sensor resolution configured via Solaar",
        &DPI_OPTIONS,
    );

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
