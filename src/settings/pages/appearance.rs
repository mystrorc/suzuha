use std::fs;
use std::path::Path;
use std::process::Command;

use super::super::button::{self, Style};
use super::super::choice;
use super::super::page::Page;
use super::super::row;
use super::super::slider::{self, Range};
use super::super::switch;
use super::super::Settings;

pub const CURSORS: [(&str, &str); 10] = [
    ("Adwaita (Default)", "Adwaita"),
    ("Bibata Modern Classic", "Bibata-Modern-Classic"),
    ("Bibata Modern Ice", "Bibata-Modern-Ice"),
    ("Bibata Modern Amber", "Bibata-Modern-Amber"),
    ("Bibata Original Classic", "Bibata-Original-Classic"),
    ("Bibata Original Ice", "Bibata-Original-Ice"),
    ("Catppuccin Mocha Dark", "catppuccin-mocha-dark-cursors"),
    ("Catppuccin Latte Light", "catppuccin-latte-light-cursors"),
    ("Nordzy Cursors", "Nordzy-cursors"),
    ("Pop OS", "Pop"),
];

pub const SIZES: [(&str, &str); 4] = [
    ("Small (24px)", "24"),
    ("Medium (32px)", "32"),
    ("Large (48px)", "48"),
    ("Extra Large (64px)", "64"),
];

pub const FONTS: [(&str, &str); 10] = [
    ("Poppins (Default)", "Poppins"),
    ("Inter", "Inter"),
    ("Adwaita Sans", "Adwaita Sans"),
    ("Cantarell", "Cantarell"),
    ("JetBrains Mono", "JetBrainsMono Nerd Font"),
    ("Fira Sans", "Fira Sans"),
    ("Roboto", "Roboto"),
    ("Ubuntu", "Ubuntu"),
    ("Noto Sans", "Noto Sans"),
    ("DejaVu Sans", "DejaVu Sans"),
];

pub const SDDM_THEMES: [(&str, &str); 7] = [
    ("Sugar Dark", "sugar-dark"),
    ("Sugar Candy", "sugar-candy"),
    ("Astronaut", "sddm-astronaut-theme"),
    ("Catppuccin Mocha Mauve", "catppuccin-mocha-mauve"),
    ("Corners", "corners"),
    ("Where Is My SDDM Theme", "where_is_my_sddm_theme"),
    ("Sway Fedora (Default)", "03-sway-fedora"),
];

pub fn build(page: &mut Page) {
    // Typography
    choice::add(
        page,
        "font_family",
        "Font family",
        "Primary typeface used across the shell",
        &FONTS,
    );

    // Mouse Pointer & Cursors
    choice::add(
        page,
        "cursor_theme",
        "Cursor theme",
        "Active mouse pointer and cursor theme",
        &CURSORS,
    );

    choice::add(
        page,
        "cursor_size",
        "Cursor size",
        "Scale of the mouse pointer in pixels",
        &SIZES,
    );

    switch::add(
        page,
        "shake_cursor",
        "Shake to find cursor",
        "Temporarily enlarge the cursor when shaken rapidly",
    );

    // Wallpaper blur and transparency
    slider::add(
        page,
        "blur_strength",
        "Blur strength",
        "Control wallpaper blur behind the wallpaper picker",
        Range {
            min: 0.0,
            max: 1.0,
            step: 0.05,
            label: slider::plain,
        },
    );

    switch::add(
        page,
        "reduce_transparency",
        "Reduce transparency",
        "Solid wallpaper picker background with no blur",
    );

    page.end_group();

    // Login Screen (SDDM Theme)
    choice::add(
        page,
        "sddm_theme",
        "Login screen theme",
        "Display manager theme for SDDM login screen",
        &SDDM_THEMES,
    );

    action(
        page,
        "Apply theme",
        "Save and activate the selected theme for SDDM",
        "Apply",
        Style::Primary,
        apply_sddm_theme,
    );

    action(
        page,
        "Preview in test mode",
        "Launch SDDM greeter in test window without logging out",
        "Preview",
        Style::Plain,
        preview_sddm_theme,
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

fn apply_sddm_theme() {
    let settings = Settings::read();
    let theme_id = settings.staged("sddm_theme");
    let content = format!("[Theme]\nCurrent={theme_id}\n");
    let _ = fs::write("/etc/sddm.conf.d/10-theme.conf", content);
}

fn preview_sddm_theme() {
    let settings = Settings::read();
    let theme_id = settings.staged("sddm_theme");
    let path = format!("/usr/share/sddm/themes/{theme_id}");
    let greeter = if Path::new("/usr/bin/sddm-greeter-qt6").exists() {
        "sddm-greeter-qt6"
    } else {
        "sddm-greeter"
    };
    let _ = Command::new(greeter)
        .arg("--test-mode")
        .arg("--theme")
        .arg(path)
        .spawn();
}
