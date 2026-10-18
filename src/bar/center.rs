use amane::{
    Audio, Button, Center, Monitor, Parent, Pointer, Rectangle, Row, Scroll, Service, Stack, Text,
    Widget, children,
};

use super::{pill, ring, system, timer, workspaces};
use crate::clock::Clock;
use crate::fonts;
use crate::overlay::Overlay;
use crate::settings::Settings;
use crate::theme::Theme;

const RING_SIZE: f32 = 24.0;
const RING_THICKNESS: f32 = 3.0;

// how far one wheel step moves a level
const STEP: i32 = 5;

pub fn view(monitor: &Monitor, theme: &Theme, width: f32) -> Row {
    let settings = Settings::read();

    let show_workspaces = settings.flag("bar_workspaces");
    let workspace_style = settings.text("workspace_style").to_string();
    let show_media = settings.flag("bar_media");
    let show_audio = settings.flag("bar_audio");
    let show_clock = settings.flag("bar_clock");
    let show_battery = settings.flag("bar_battery");
    let show_memory = settings.flag("bar_memory");

    // the clock reads the settings again, so this read ends first
    drop(settings);

    let mut items: Vec<Box<dyn Widget>> = Vec::new();

    if show_workspaces {
        items.push(Box::new(workspaces::workspaces_strip(
            monitor,
            theme,
            &workspace_style,
        )));
    }

    if show_media {
        if let Some(media) = workspaces::media(theme) {
            items.push(Box::new(media));
        }
    }

    if show_audio {
        items.push(Box::new(audio(theme)));
    }

    if let Some(timer) = timer::view(theme) {
        items.push(Box::new(timer));
    }

    if show_clock {
        items.push(Box::new(clock(theme)));
    }

    if show_battery {
        if let Some(battery) = system::battery(theme) {
            items.push(Box::new(battery));
        }
    }

    if show_memory {
        items.push(Box::new(system::memory(theme)));
    }

    Row::new(items)
        .width(width)
        .height(Parent)
        .gap(8.0)
        .justify(Center)
        .align(Center)
}

// Volume pill with speaker icon inside the ring and volume percentage to the side
fn audio(theme: &Theme) -> Rectangle {
    let audio = Audio::read();
    let volume = audio.volume();
    let muted = audio.muted();

    let speaker_color = if muted {
        theme.muted_text
    } else {
        theme.accent
    };

    let text_color = if muted {
        theme.muted_text
    } else {
        theme.text
    };

    let icon_glyph = if muted || volume == 0 {
        "󰝟"
    } else if volume < 35 {
        "󰕿"
    } else if volume < 70 {
        "󰖀"
    } else {
        "󰕾"
    };

    let speaker_value = f32::from(volume) / 100.0;

    let ring = ring::view(RING_SIZE, RING_THICKNESS, speaker_value, speaker_color, theme.border);

    let icon = Rectangle::new()
        .width(RING_SIZE)
        .height(RING_SIZE)
        .align_child(Center, Center)
        .child(Text::new(icon_glyph).size(11.0).font(fonts::MATERIAL).color(text_color));

    let ring_with_icon = Stack::new(children![ring, icon]);

    let text = format!("{volume}%");
    let text_w = pill::text_width(&text, 14.0);

    let ind = Row::new(children![ring_with_icon, pill::label(&text, 14.0, text_color)])
        .gap(6.0)
        .align(Center);

    let width = RING_SIZE + 6.0 + text_w + 16.0;

    pill::view(width, theme.surface)
        .cursor(Pointer)
        .on_scroll(scroll_volume)
        .on_click(|button| match button {
            Button::Left => Overlay::toggle_control_center(),
            Button::Right => {
                let is_eq_open = {
                    let o = Overlay::read();
                    o.control_center.shown && o.center_tab == 1 && o.control_tab == 1
                };
                if is_eq_open {
                    Overlay::toggle_control_center();
                } else {
                    Overlay::open_control_tab(1);
                }
            }
            _ => {}
        })
        .align_child(Center, Center)
        .child(ind)
}

// Supports showing just time, just date, or both inside a surface pill
fn clock(theme: &Theme) -> Rectangle {
    let clock = Clock::read();
    let settings = Settings::read();

    let seconds = settings.flag("clock_seconds");
    let display = settings.text("clock_display");

    let text = match display {
        "time" => clock.time(seconds),
        "date" => String::from(clock.date()),
        _ => format!("{}  •  {}", clock.time(seconds), clock.date()),
    };

    drop(settings);

    let text_width = pill::text_width(&text, 14.0);

    pill::view(text_width + 24.0, theme.surface)
        .align_child(Center, Center)
        .child(pill::label(&text, 14.0, theme.text))
}

fn scroll_volume(scroll: Scroll) {
    let volume = i32::from(Audio::read().volume());

    Audio::set_volume(step(volume, scroll));
}

// wheel up is a negative y, which turns the level up
fn step(level: i32, scroll: Scroll) -> u8 {
    let change = if scroll.y < 0.0 { STEP } else { -STEP };

    (level + change).clamp(0, 100) as u8
}
