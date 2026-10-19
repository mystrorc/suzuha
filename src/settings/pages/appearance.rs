use super::super::choice;
use super::super::page::Page;
use super::super::slider::{self, Range};
use super::super::switch;

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


pub fn build(page: &mut Page) {
    // Window Appearance (Corners, Borders, Gaps, Shadows)
    slider::add(
        page,
        "window_corner_radius",
        "Window corner radius",
        "Rounded corner radius clipped across all windows and borders",
        Range {
            min: 0.0,
            max: 24.0,
            step: 2.0,
            label: slider::pixels,
        },
    );

    slider::add(
        page,
        "window_border_width",
        "Window border width",
        "Thickness of the rounded border around windows",
        Range {
            min: 0.0,
            max: 6.0,
            step: 1.0,
            label: slider::pixels,
        },
    );

    switch::add(
        page,
        "window_inactive_border",
        "Inactive window border",
        "Show a subtle rounded border around unfocused windows",
    );

    slider::add(
        page,
        "window_gaps",
        "Window gaps",
        "Spacing between tiled windows on the desktop",
        Range {
            min: 0.0,
            max: 24.0,
            step: 2.0,
            label: slider::pixels,
        },
    );

    switch::add(
        page,
        "window_shadows",
        "Window drop shadows",
        "Render soft rounded drop shadows behind windows",
    );

    page.end_group();

    // Typography & Cursors
    choice::add(
        page,
        "font_family",
        "Font family",
        "Primary typeface used across the shell",
        &FONTS,
    );

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
        "Solid wallpaper picker and app backgrounds with no blur",
    );

    page.end_group();

}
