use amane::{
    Audio, Button, Center, Parent, Pointer, Rectangle, Row, Scroll, Service, Widget,
    children,
};

use super::{pill, ring, timer};
use crate::clock::Clock;
use crate::settings::Settings;
use crate::overlay::Overlay;
use crate::theme::Theme;

const RING_SIZE: f32 = 18.0;
const RING_THICKNESS: f32 = 4.0;

// how far one wheel step moves a level
const STEP: i32 = 5;

pub fn view(theme: &Theme, width: f32) -> Row {
    let settings = Settings::read();

    let show_audio = settings.flag("bar_audio");
    let show_clock = settings.flag("bar_clock");

    // the clock reads the settings again, so this read ends first
    drop(settings);

    let mut items: Vec<Box<dyn Widget>> = Vec::new();

    if show_audio {
        items.push(Box::new(audio(theme)));
    }

    if let Some(timer) = timer::view(theme) {
        items.push(Box::new(timer));
    }

    if show_clock {
        items.push(Box::new(clock(theme)));
    }

    Row::new(items)
        .width(width)
        .height(Parent)
        .gap(2.0)
        .justify(Center)
        .align(Center)
}

// the speaker ring only on the bar (microphone remains in pulldown tab), scroll to change level, click to open control center
fn audio(theme: &Theme) -> Rectangle {
    let audio = Audio::read();

    let speaker_color = if audio.muted() {
        theme.muted_text
    } else {
        theme.accent
    };

    let speaker_value = f32::from(audio.volume()) / 100.0;

    let speaker = Rectangle::new()
        .width(RING_SIZE)
        .height(RING_SIZE)
        .on_scroll(scroll_volume)
        .child(ring::view(RING_SIZE, RING_THICKNESS, speaker_value, speaker_color, theme.border));

    // one ring plus 16px of padding
    let width = RING_SIZE + 16.0;

    pill::view(width, theme.surface)
        .cursor(Pointer)
        .on_click(|button| match button {
            Button::Left => Overlay::toggle_control_center(),
            Button::Right => {
                if Overlay::read().control_center.shown && Overlay::read().control_tab == 1 {
                    Overlay::toggle_control_center();
                } else {
                    Overlay::open_control_tab(1);
                }
            }
            _ => {}
        })
        .align_child(Center, Center)
        .child(speaker)
}

// "09:05 PM  •  Tuesday, 29 Sep 2026", plain text with 8px in front, no pill
fn clock(theme: &Theme) -> Row {
    let clock = Clock::read();

    let seconds = Settings::read().flag("clock_seconds");

    let text = format!("{}  •  {}", clock.time(seconds), clock.date());

    Row::new(children![
        Rectangle::new().width(8.0).height(1.0),
        pill::label(&text, 15.0, theme.text),
    ])
    .align(Center)
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
