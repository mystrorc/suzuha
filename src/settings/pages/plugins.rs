use super::super::button::{self, Style};
use super::super::page::Page;
use super::super::row;
use crate::overlay::Overlay;

pub fn build(page: &mut Page) {
    // Group 1: Converted Omarchy Plugins
    action(
        page,
        "System Monitor",
        "Omarchy cynricantao.system-monitor port • Attached to RAM pill (Memory, CPU, Swap)",
        "Open Flyout",
        Style::Plain,
        || {
            Overlay::toggle_sysmon();
        },
    );

    action(
        page,
        "Audio & Equalizer",
        "Omarchy ssupt.audio-control port • Integrated into Control Center Audio & 10-band EQ tab",
        "Open Audio",
        Style::Plain,
        || {
            Overlay::open_control_tab(1);
        },
    );

    action(
        page,
        "Power & Devices",
        "Battery monitor • Laptop, keyboard, mouse & headphones (Attached to Battery pill)",
        "Open Flyout",
        Style::Plain,
        || {
            Overlay::toggle_power_devices();
        },
    );

    action(
        page,
        "Monitor Layout",
        "Omarchy cjgarcia1229.monitor-layout port • Drag & drop display editor with snap & scaling",
        "Open Editor",
        Style::Plain,
        || {
            let _ = std::process::Command::new("python3")
                .arg("/home/rcrout/.config/amane/plugins/monitor-layout/monitor_layout.py")
                .spawn();
        },
    );

    page.end_group();

    // Group 2: Equalizer Engine
    action(
        page,
        "Equalizer Bypass",
        "Toggle real-time 10-band parametric equalizer on or off",
        "Toggle Bypass",
        Style::Primary,
        || {
            crate::overlay::control_center::audio_eq::toggle_bypass();
        },
    );

    action(
        page,
        "Reset Equalizer",
        "Reset all frequency bands to Flat (0.0 dB)",
        "Reset Flat",
        Style::Plain,
        || {
            crate::overlay::control_center::audio_eq::set_preset("Flat");
        },
    );

    page.end_group();

    // Group 3: Source Code & Configuration
    action(
        page,
        "Plugin Source Code",
        "Converted Rust source files located in ~/.config/amane/src/",
        "Open Folder",
        Style::Plain,
        || {
            let _ = std::process::Command::new("xdg-open")
                .arg("/home/rcrout/.config/amane/src")
                .spawn();
        },
    );

    action(
        page,
        "Plugin Directory",
        "Omarchy & custom plugins located in ~/.config/amane/plugins/",
        "Open Folder",
        Style::Plain,
        || {
            let _ = std::process::Command::new("xdg-open")
                .arg("/home/rcrout/.config/amane/plugins")
                .spawn();
        },
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
