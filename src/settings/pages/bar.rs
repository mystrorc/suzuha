use super::super::page::Page;
use super::super::slider::{self, Range};
use super::super::{choice, switch};

pub fn build(page: &mut Page) {
    choice::add(
        page,
        "bar_position",
        "Position",
        "Place the bar along the top or bottom edge of the screen",
        &[("Top", "top"), ("Bottom", "bottom")],
    );

    slider::add(
        page,
        "bar_height",
        "Bar height",
        "Height in screen pixels reserved for the bar",
        Range {
            min: 32.0,
            max: 56.0,
            step: 2.0,
            label: slider::pixels,
        },
    );

    switch::add(
        page,
        "bar_auto_hide",
        "Auto hide",
        "Reveal the bar only while the pointer rests along the screen edge",
    );

    choice::add(
        page,
        "workspace_style",
        "Workspace style",
        "Choose how workspace slots appear in the bar strip",
        &[("Pill", "pill"), ("Dots", "dots"), ("Numbers", "numbers")],
    );

    page.end_group();

    switch::add(page, "bar_logo", "Distro logo", "Show the applications launcher");
    switch::add(page, "bar_workspaces", "Workspaces", "Show the workspace indicator");
    switch::add(page, "bar_workspace_name", "Workspace name", "Show the active workspace label");
    switch::add(page, "bar_audio", "Audio controls", "Show output and microphone rings");
    switch::add(page, "bar_media", "Media", "Show current media information when playing");
    switch::add(page, "bar_clock", "Clock and date", "Show time and/or date in the center section");
    switch::add(page, "bar_battery", "Battery", "Show battery status when unplugged");
    switch::add(page, "bar_memory", "Memory", "Show memory usage");

    switch::add(
        page,
        "bar_tray",
        "System tray group",
        "Show notifications, network, and Bluetooth shortcuts",
    );

    switch::add(
        page,
        "bar_power",
        "Power menu button",
        "Show power menu button in the system tray",
    );

    page.end_group();

    choice::add(
        page,
        "clock_display",
        "Clock format",
        "Show just the time, just the date, or both in the bar",
        &[("Both", "both"), ("Time only", "time"), ("Date only", "date")],
    );
    switch::add(page, "clock_24_hour", "24-hour clock", "Use 24-hour time instead of AM/PM");
    switch::add(page, "clock_seconds", "Show seconds", "Show seconds in the bar clock");

    page.end_group();
}
