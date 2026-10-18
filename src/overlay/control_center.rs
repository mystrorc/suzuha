mod art;
pub mod audio_eq;
mod cava;
mod fader;
mod media;
mod player;
mod visualizer;
pub mod wave;

use amane::{Audio, Center, Column, Padding, Pointer, Rectangle, Row, Service, Text, Weight, Widget, children};

use crate::fonts;

use super::{Overlay, PanelView, Region, power_devices, sysmon};
use crate::liquid::{self, Blob};
use crate::theme::Theme;

const MAX_WIDTH: f32 = 780.0;

const TOP_BAR_HEIGHT: f32 = 36.0;
const TOP_BAR_GAP: f32 = 12.0;
const CARD_HEIGHT: f32 = 240.0;

// 288 tall on screen (36px top-level tab bar + 12px gap + 240px content), plus what hides past the top edge
const INNER_HEIGHT: f32 = TOP_BAR_HEIGHT + TOP_BAR_GAP + CARD_HEIGHT;
const HEIGHT: f32 = INNER_HEIGHT + EDGE_OVERLAP + 34.0;
const RADIUS: f32 = 30.0;

/*
 * the panel reaches past the top edge by more than its corner radius, so
 * its sides meet the edge straight; with the round corners on screen the
 * melt narrows once it stops moving
 */
const EDGE_OVERLAP: f32 = 40.0;

// how far past its own height it slides to let go of the edge
const HIDDEN_MARGIN: f32 = 32.0;

const PADDING: Padding = Padding {
    top: EDGE_OVERLAP + 16.0,
    right: 18.0,
    bottom: 18.0,
    left: 18.0,
};

const GAP: f32 = 14.0;

const FADERS_WIDTH: f32 = 120.0;
const CARD_RADIUS: f32 = 20.0;

const VOLUME_ICON: &str = "󱕾";
const MICROPHONE_ICON: &str = "󰍬";

// none while fully hidden
pub fn view(overlay: &Overlay, theme: &Theme, screen: Region) -> Option<PanelView> {
    let progress = overlay.control_center.progress.value();

    if progress <= 0.001 {
        return None;
    }

    let width = MAX_WIDTH.min(screen.width - 64.0);

    let x = (screen.width - width) / 2.0;

    let hidden = -(HEIGHT + HIDDEN_MARGIN);

    let y = -EDGE_OVERLAP + hidden * (1.0 - progress);

    let blob = Blob::new(x, y, width, HEIGHT)
        .radius(RADIUS)
        .child(content(overlay, theme, width));

    let input = Region {
        x,
        y,
        width,
        height: HEIGHT,
    };

    // fully out it hangs from the top edge, and its melted corners spread a little past its sides
    let reach = Region {
        x: x - liquid::CONNECTION,
        y: 0.0,
        width: width + liquid::CONNECTION * 2.0,
        height: -EDGE_OVERLAP + HEIGHT + liquid::CONNECTION,
    };

    Some(PanelView { blob, input, reach })
}

// Higher-level tabs at the top of the middle pulldown: System Monitor / Media / Battery
fn section_tabs(overlay: &Overlay, theme: &Theme, width: f32) -> Row {
    let sysmon_btn = section_tab_button(
        theme,
        "󰍛",
        "System Monitor",
        overlay.center_tab == 0,
        || Overlay::set_center_tab(0),
    );
    let media_btn = section_tab_button(
        theme,
        "󰎆",
        "Media",
        overlay.center_tab == 1,
        || Overlay::set_center_tab(1),
    );
    let battery_btn = section_tab_button(
        theme,
        "󰁹",
        "Battery",
        overlay.center_tab == 2,
        || Overlay::set_center_tab(2),
    );

    let strip = Rectangle::new()
        .width(420.0)
        .height(TOP_BAR_HEIGHT)
        .radius(14.0)
        .fill(theme.surface)
        .padding(Padding {
            top: 4.0,
            right: 4.0,
            bottom: 4.0,
            left: 4.0,
        })
        .align_child(Center, Center)
        .child(
            Row::new(children![sysmon_btn, media_btn, battery_btn])
                .gap(6.0)
                .align(Center),
        );

    Row::new(children![strip])
        .width(width)
        .height(TOP_BAR_HEIGHT)
        .justify(Center)
        .align(Center)
}

fn section_tab_button(
    theme: &Theme,
    icon: &str,
    label: &str,
    active: bool,
    on_click: impl Fn() + 'static,
) -> Rectangle {
    let fill = if active {
        theme.accent
    } else {
        amane::Color::TRANSPARENT
    };
    let text_color = if active {
        theme.on_accent
    } else {
        theme.secondary_text
    };
    let icon_color = if active {
        theme.on_accent
    } else {
        theme.accent
    };

    let icon_w = Text::new(icon)
        .size(14.0)
        .font(fonts::NERD)
        .tight()
        .color(icon_color);

    let label_w = Text::new(label)
        .size(12.0)
        .font(fonts::BODY)
        .weight(if active { Weight::SemiBold } else { Weight::Medium })
        .color(text_color);

    let row = Row::new(children![icon_w, label_w]).gap(6.0).align(Center);

    Rectangle::new()
        .width(134.0)
        .height(28.0)
        .radius(10.0)
        .fill(fill)
        .cursor(Pointer)
        .on_click(move |_| on_click())
        .align_child(Center, Center)
        .child(row)
}

// Sub-tabs underneath Media: Now Playing / Audio & EQ
pub fn tabs(overlay: &Overlay, theme: &Theme) -> Row {
    let now_playing = tab_button(
        theme,
        "󰎆",
        "Now Playing",
        overlay.control_tab == 0,
        || Overlay::set_control_tab(0),
    );
    let audio_eq = tab_button(
        theme,
        "󱓃",
        "Audio & EQ",
        overlay.control_tab == 1,
        || Overlay::set_control_tab(1),
    );

    Row::new(children![now_playing, audio_eq])
        .gap(6.0)
        .align(Center)
}

fn tab_button(
    theme: &Theme,
    icon: &str,
    label: &str,
    active: bool,
    on_click: impl Fn() + 'static,
) -> Rectangle {
    let fill = if active {
        theme.selected_surface
    } else {
        amane::Color::TRANSPARENT
    };
    let text_color = if active {
        theme.text
    } else {
        theme.muted_text
    };

    let icon_w = Text::new(icon)
        .size(13.0)
        .font(fonts::NERD)
        .tight()
        .color(if active { theme.accent } else { theme.secondary_text });

    let label_w = Text::new(label)
        .size(12.0)
        .font(fonts::BODY)
        .weight(if active { Weight::SemiBold } else { Weight::Regular })
        .color(text_color);

    let row = Row::new(children![icon_w, label_w]).gap(6.0).align(Center);
    let width = label.chars().count() as f32 * 7.5 + 38.0;

    Rectangle::new()
        .width(width)
        .height(26.0)
        .radius(8.0)
        .fill(fill)
        .padding(Padding {
            top: 4.0,
            bottom: 4.0,
            left: 10.0,
            right: 10.0,
        })
        .cursor(Pointer)
        .on_click(move |_| on_click())
        .align_child(Center, Center)
        .child(row)
}

fn content(overlay: &Overlay, theme: &Theme, width: f32) -> Rectangle {
    let inner_width = width - PADDING.left - PADDING.right;

    let top_tabs = section_tabs(overlay, theme, inner_width);

    let body: Box<dyn Widget> = match overlay.center_tab {
        0 => Box::new(sysmon::embedded_view(theme, inner_width, CARD_HEIGHT)),
        2 => Box::new(power_devices::embedded_view(theme, inner_width, CARD_HEIGHT)),
        _ => {
            let card_width = inner_width - FADERS_WIDTH - GAP;

            let right_card: Box<dyn Widget> = match overlay.control_tab {
                1 => Box::new(audio_eq::view(overlay, theme, card_width, CARD_HEIGHT)),
                _ => Box::new(media::view(overlay, theme, card_width, CARD_HEIGHT)),
            };

            Box::new(Row::new(vec![Box::new(faders(theme)) as Box<dyn Widget>, right_card]).gap(GAP))
        }
    };

    let column = Column::new(vec![Box::new(top_tabs) as Box<dyn Widget>, body])
        .width(inner_width)
        .gap(TOP_BAR_GAP);

    Rectangle::new()
        .width(width)
        .height(HEIGHT)
        .padding(PADDING)
        .on_hover(Overlay::hover_control_center)
        .child(column)
}

fn faders(theme: &Theme) -> Rectangle {
    let audio = Audio::read();

    let volume = fader::Fader {
        icon: VOLUME_ICON,
        level: audio.volume(),
        muted: audio.muted(),
        set: Audio::set_volume,
        toggle_mute: Audio::toggle_mute,
    };

    let microphone = fader::Fader {
        icon: MICROPHONE_ICON,
        level: audio.microphone_volume(),
        muted: audio.microphone_muted(),
        set: Audio::set_microphone_volume,
        toggle_mute: Audio::toggle_microphone_mute,
    };

    let row = Row::new(children![
        fader::view(volume, theme, CARD_HEIGHT - 12.0),
        fader::view(microphone, theme, CARD_HEIGHT - 12.0),
    ])
    .gap(4.0);

    Rectangle::new()
        .width(FADERS_WIDTH)
        .height(CARD_HEIGHT)
        .radius(CARD_RADIUS)
        .fill(theme.surface)
        .align_child(Center, Center)
        .child(row)
}

// "toggle", "show" or "hide"
pub fn ipc(arguments: &[String]) -> String {
    let command = arguments.first().map(String::as_str).unwrap_or("toggle");

    let shown = Overlay::read().control_center.shown && Overlay::read().center_tab == 1;

    let wanted = match command {
        "show" => true,
        "hide" => false,
        _ => !shown,
    };

    if wanted != shown {
        Overlay::toggle_control_center();
    }

    String::from("ok")
}
