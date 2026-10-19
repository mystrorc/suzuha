use amane::{
    Center, Column, Padding, Pointer, Rectangle, Row, Service, Start, Text, TextInput,
    Weight, children,
};

use super::super::button::{self, Style};
use super::super::choice;
use super::super::field;
use super::super::page::Page;
use super::super::row;
use super::super::slider::{self, Range};
use super::super::switch;
use super::super::{Settings, high_surface};
use super::kanata::{
    self, ACTION_PRESETS, REMAPPER, RemapRule, TRIGGER_PRESETS, fix_macro_digits, text_to_kanata_macro,
};
use crate::fonts;
use crate::theme::Theme;

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

pub fn cycle_kanata_device(delta: isize) {
    let mut state = REMAPPER.lock().unwrap();
    if state.devices.is_empty() {
        return;
    }
    let count = state.devices.len() as isize;
    let next = (state.selected_device_idx as isize + delta).rem_euclid(count) as usize;
    state.selected_device_idx = next;
    state.profile.device = state.devices[next].id.clone();
}

pub fn cycle_kanata_trigger(delta: isize) {
    let mut state = REMAPPER.lock().unwrap();
    let count = TRIGGER_PRESETS.len() as isize;
    let next = (state.trigger_preset_idx as isize + delta).rem_euclid(count) as usize;
    state.trigger_preset_idx = next;
}

pub fn cycle_kanata_action(delta: isize) {
    let mut state = REMAPPER.lock().unwrap();
    let count = ACTION_PRESETS.len() as isize;
    let next = (state.action_preset_idx as isize + delta).rem_euclid(count) as usize;
    state.action_preset_idx = next;
}

pub fn remove_kanata_rule(index: usize) {
    let mut state = REMAPPER.lock().unwrap();
    if index < state.profile.rules.len() {
        state.profile.rules.remove(index);
    }
}

pub fn add_kanata_rule() {
    let mut state = REMAPPER.lock().unwrap();
    let trigger = TRIGGER_PRESETS[state.trigger_preset_idx].1.to_string();
    let action_preset = ACTION_PRESETS[state.action_preset_idx];

    let action = if action_preset.1 == "__TEXT__" {
        if state.custom_text.trim().is_empty() {
            "macro h e l l o".to_string()
        } else {
            text_to_kanata_macro(&state.custom_text)
        }
    } else if action_preset.1 == "__CUSTOM__" {
        if state.custom_syntax.trim().is_empty() {
            "tap-hold 200 200 esc lctl".to_string()
        } else {
            state.custom_syntax.clone()
        }
    } else {
        action_preset.1.to_string()
    };

    let desc = if !state.rule_desc.trim().is_empty() {
        state.rule_desc.clone()
    } else if action_preset.1 == "__TEXT__" {
        format!("Type \"{}\"", state.custom_text)
    } else {
        format!("{} to {}", TRIGGER_PRESETS[state.trigger_preset_idx].0, action_preset.0)
    };

    state.profile.rules.push(RemapRule {
        trigger,
        action: fix_macro_digits(&action),
        description: desc,
    });

    state.custom_text.clear();
    state.custom_syntax.clear();
    state.rule_desc.clear();
}

pub fn apply_kanata_config() {
    let mut state = REMAPPER.lock().unwrap();
    match kanata::apply_and_restart(&state.profile) {
        Ok(msg) => {
            state.status_message = msg;
            state.is_active = kanata::is_service_active();
        }
        Err(err) => {
            state.status_message = format!("Error: {err}");
            state.is_active = false;
        }
    }
}

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

    let current_dev = String::from(Settings::read().staged("input_pointer_device"));

    match current_dev.as_str() {
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

    // =========================================================================
    // Tools / Input Remapper (Kanata Engine Integration) Section
    // =========================================================================
    let kanata_state = REMAPPER.lock().unwrap().clone();
    let theme = page.theme;
    let width = page.width;

    // Header Card
    let header_widget = build_kanata_header(theme, width, &kanata_state);
    page.row(52.0, header_widget);
    page.end_group();

    // Target Input Device
    let dev_name = kanata_state
        .devices
        .get(kanata_state.selected_device_idx)
        .map(|d| d.display_name.as_str())
        .unwrap_or("All Devices (Default)");
    let dev_detail = kanata_state
        .devices
        .get(kanata_state.selected_device_idx)
        .map(|d| d.description.as_str())
        .unwrap_or("Applies rules globally to all keyboards");
    let dev_selector = build_device_selector(theme, dev_name);
    let dev_row = row::view(
        theme,
        width,
        row::HEIGHT,
        "Target input device",
        dev_detail,
        dev_selector,
        280.0,
    );
    page.row(row::HEIGHT, dev_row);

    // Configured Remap Rules
    for (idx, r) in kanata_state.profile.rules.iter().enumerate() {
        let rule_card = build_rule_row(theme, width, idx, r);
        page.row(row::HEIGHT, rule_card);
    }

    if kanata_state.profile.rules.is_empty() {
        let empty_text = Text::new("No remapping rules configured. Add a rule below.")
            .size(13.0)
            .font(fonts::BODY)
            .color(theme.muted_text);
        let empty_row = Rectangle::new()
            .width(width)
            .height(row::HEIGHT)
            .align_child(Start, Center)
            .padding(Padding { top: 0.0, bottom: 0.0, left: 20.0, right: 20.0 })
            .child(empty_text);
        page.row(row::HEIGHT, empty_row);
    }

    page.end_group();

    // Add New Remapping Rule Group
    let trigger_name = TRIGGER_PRESETS[kanata_state.trigger_preset_idx].0;
    let trigger_ctrl = build_trigger_selector(theme, trigger_name);
    let trigger_row = row::view(
        theme,
        width,
        row::HEIGHT,
        "Trigger key / button",
        "Hardware key or modifier to capture and intercept",
        trigger_ctrl,
        280.0,
    );
    page.row(row::HEIGHT, trigger_row);

    let action_name = ACTION_PRESETS[kanata_state.action_preset_idx].0;
    let action_ctrl = build_action_selector(theme, action_name);
    let action_row = row::view(
        theme,
        width,
        row::HEIGHT,
        "Remapped action",
        ACTION_PRESETS[kanata_state.action_preset_idx].2,
        action_ctrl,
        280.0,
    );
    page.row(row::HEIGHT, action_row);

    let is_text = ACTION_PRESETS[kanata_state.action_preset_idx].1 == "__TEXT__";
    let is_custom = ACTION_PRESETS[kanata_state.action_preset_idx].1 == "__CUSTOM__";

    if is_text {
        let text_input = TextInput::new("kanata_text_input")
            .size(13.0)
            .color(theme.text)
            .placeholder("e.g. Hello World or email@example.com")
            .on_change(|val| {
                if let Ok(mut state) = REMAPPER.lock() {
                    state.custom_text = val;
                }
            });
        let field_box = build_field_box(theme, text_input);
        let text_row = row::view(
            theme,
            width,
            row::HEIGHT,
            "Text to type",
            "String expanded character-by-character into hardware keypresses",
            field_box,
            280.0,
        );
        page.row(row::HEIGHT, text_row);
    }

    if is_custom {
        let syntax_input = TextInput::new("kanata_syntax_input")
            .size(13.0)
            .color(theme.text)
            .placeholder("e.g. tap-hold 200 200 esc lctl")
            .on_change(|val| {
                if let Ok(mut state) = REMAPPER.lock() {
                    state.custom_syntax = val;
                }
            });
        let field_box = build_field_box(theme, syntax_input);
        let syntax_row = row::view(
            theme,
            width,
            row::HEIGHT,
            "Custom Kanata syntax",
            "Low-level Kanata action string (e.g. tap-hold, macro, layer)",
            field_box,
            280.0,
        );
        page.row(row::HEIGHT, syntax_row);
    }

    let desc_input = TextInput::new("kanata_desc_input")
        .size(13.0)
        .color(theme.text)
        .placeholder("e.g. Scroll Lock to Volume Up (+5%)")
        .on_change(|val| {
            if let Ok(mut state) = REMAPPER.lock() {
                state.rule_desc = val;
            }
        });
    let desc_box = build_field_box(theme, desc_input);
    let desc_row = row::view(
        theme,
        width,
        row::HEIGHT,
        "Rule description",
        "Optional human-readable label for this remapping rule",
        desc_box,
        280.0,
    );
    page.row(row::HEIGHT, desc_row);

    // Add Rule button
    let add_btn = button::view(theme, "+ Add Remapping Rule", Style::Plain, true, add_kanata_rule);
    let add_row = row::view(
        theme,
        width,
        row::HEIGHT,
        "Add rule to active profile",
        "Stage the configured key remapping rule into the rule list",
        add_btn,
        button::width("+ Add Remapping Rule"),
    );
    page.row(row::HEIGHT, add_row);

    page.end_group();

    // Apply Kanata Group
    let apply_btn = button::view(theme, "Apply & Reload Remapper", Style::Primary, true, apply_kanata_config);
    let apply_row = row::view(
        theme,
        width,
        row::HEIGHT,
        "Save and reload Kanata daemon",
        "Writes ~/.config/kanata/kanata.kbd and restarts the background service",
        apply_btn,
        button::width("Apply & Reload Remapper"),
    );
    page.row(row::HEIGHT, apply_row);
    page.end_group();
}

fn build_kanata_header(theme: &Theme, width: f32, state: &kanata::RemapperState) -> Rectangle {
    let icon = Text::new("󰌌").size(16.0).font(fonts::NERD).tight().color(theme.accent);
    let title = Text::new("Input Remapper (Keys & Buttons)")
        .size(15.0)
        .font(fonts::BODY)
        .weight(Weight::SemiBold)
        .color(theme.text);

    let status_color = if state.is_active { theme.accent } else { theme.secondary_text };
    let dot = Text::new(if state.is_active { "●" } else { "○" })
        .size(9.0)
        .font(fonts::BODY)
        .color(status_color);
    let subtitle = Text::new(&state.status_message)
        .size(12.0)
        .font(fonts::BODY)
        .color(theme.muted_text)
        .elide();

    let text_col = Column::new(children![
        Row::new(children![icon, title]).gap(8.0).align(Center),
        Row::new(children![dot, subtitle]).gap(6.0).align(Center),
    ])
    .gap(2.0);

    Rectangle::new()
        .width(width)
        .height(52.0)
        .padding(Padding {
            top: 6.0,
            bottom: 6.0,
            left: 18.0,
            right: 18.0,
        })
        .align_child(Start, Center)
        .child(text_col)
}

fn build_device_selector(theme: &Theme, dev_name: &str) -> Rectangle {
    let prev_btn = Rectangle::new()
        .width(28.0)
        .height(28.0)
        .radius(8.0)
        .fill(high_surface(theme))
        .cursor(Pointer)
        .on_click(|_| cycle_kanata_device(-1))
        .align_child(Center, Center)
        .child(Text::new("󰅁").size(14.0).font(fonts::NERD).tight().color(theme.text));

    let next_btn = Rectangle::new()
        .width(28.0)
        .height(28.0)
        .radius(8.0)
        .fill(high_surface(theme))
        .cursor(Pointer)
        .on_click(|_| cycle_kanata_device(1))
        .align_child(Center, Center)
        .child(Text::new("󰅂").size(14.0).font(fonts::NERD).tight().color(theme.text));

    let label_w = 280.0 - 56.0 - 16.0;
    let label = Text::new(dev_name)
        .size(13.0)
        .font(fonts::BODY)
        .weight(Weight::SemiBold)
        .color(theme.accent)
        .elide();

    let center_box = Rectangle::new()
        .width(label_w)
        .height(28.0)
        .radius(8.0)
        .fill(high_surface(theme))
        .padding(Padding { top: 0.0, bottom: 0.0, left: 10.0, right: 10.0 })
        .align_child(Center, Center)
        .child(label);

    let row = Row::new(children![prev_btn, center_box, next_btn])
        .gap(8.0)
        .align(Center);

    Rectangle::new()
        .width(280.0)
        .height(row::CONTROL_HEIGHT)
        .align_child(Start, Center)
        .child(row)
}

fn build_trigger_selector(theme: &Theme, trigger_name: &str) -> Rectangle {
    let prev_btn = Rectangle::new()
        .width(28.0)
        .height(28.0)
        .radius(8.0)
        .fill(high_surface(theme))
        .cursor(Pointer)
        .on_click(|_| cycle_kanata_trigger(-1))
        .align_child(Center, Center)
        .child(Text::new("󰅁").size(14.0).font(fonts::NERD).tight().color(theme.text));

    let next_btn = Rectangle::new()
        .width(28.0)
        .height(28.0)
        .radius(8.0)
        .fill(high_surface(theme))
        .cursor(Pointer)
        .on_click(|_| cycle_kanata_trigger(1))
        .align_child(Center, Center)
        .child(Text::new("󰅂").size(14.0).font(fonts::NERD).tight().color(theme.text));

    let label_w = 280.0 - 56.0 - 16.0;
    let label = Text::new(trigger_name)
        .size(13.0)
        .font(fonts::BODY)
        .weight(Weight::SemiBold)
        .color(theme.accent)
        .elide();

    let center_box = Rectangle::new()
        .width(label_w)
        .height(28.0)
        .radius(8.0)
        .fill(high_surface(theme))
        .padding(Padding { top: 0.0, bottom: 0.0, left: 10.0, right: 10.0 })
        .align_child(Center, Center)
        .child(label);

    let row = Row::new(children![prev_btn, center_box, next_btn])
        .gap(8.0)
        .align(Center);

    Rectangle::new()
        .width(280.0)
        .height(row::CONTROL_HEIGHT)
        .align_child(Start, Center)
        .child(row)
}

fn build_action_selector(theme: &Theme, action_name: &str) -> Rectangle {
    let prev_btn = Rectangle::new()
        .width(28.0)
        .height(28.0)
        .radius(8.0)
        .fill(high_surface(theme))
        .cursor(Pointer)
        .on_click(|_| cycle_kanata_action(-1))
        .align_child(Center, Center)
        .child(Text::new("󰅁").size(14.0).font(fonts::NERD).tight().color(theme.text));

    let next_btn = Rectangle::new()
        .width(28.0)
        .height(28.0)
        .radius(8.0)
        .fill(high_surface(theme))
        .cursor(Pointer)
        .on_click(|_| cycle_kanata_action(1))
        .align_child(Center, Center)
        .child(Text::new("󰅂").size(14.0).font(fonts::NERD).tight().color(theme.text));

    let label_w = 280.0 - 56.0 - 16.0;
    let label = Text::new(action_name)
        .size(13.0)
        .font(fonts::BODY)
        .weight(Weight::SemiBold)
        .color(theme.accent)
        .elide();

    let center_box = Rectangle::new()
        .width(label_w)
        .height(28.0)
        .radius(8.0)
        .fill(high_surface(theme))
        .padding(Padding { top: 0.0, bottom: 0.0, left: 10.0, right: 10.0 })
        .align_child(Center, Center)
        .child(label);

    let row = Row::new(children![prev_btn, center_box, next_btn])
        .gap(8.0)
        .align(Center);

    Rectangle::new()
        .width(280.0)
        .height(row::CONTROL_HEIGHT)
        .align_child(Start, Center)
        .child(row)
}

fn build_field_box(theme: &Theme, input: TextInput) -> Rectangle {
    Rectangle::new()
        .width(280.0)
        .height(row::CONTROL_HEIGHT)
        .radius(10.0)
        .fill(high_surface(theme))
        .padding(Padding {
            top: 0.0,
            bottom: 0.0,
            left: 12.0,
            right: 12.0,
        })
        .align_child(Start, Center)
        .child(input)
}

fn build_rule_row(theme: &Theme, width: f32, idx: usize, r: &RemapRule) -> Rectangle {
    let remove_btn = button::view(theme, "Remove", Style::Plain, true, move || {
        remove_kanata_rule(idx);
    });

    let mapping_text = format!("{}  ➜  {}", r.trigger, r.action);

    row::view(
        theme,
        width,
        row::HEIGHT,
        &r.description,
        &mapping_text,
        remove_btn,
        button::width("Remove"),
    )
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
