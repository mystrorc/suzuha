use amane::{
    Button, Center, Color, Column, End, Padding, Pointer, Rectangle, Row, Service, Stack, Start, Text,
    Weight, Widget, children,
};

use super::{Overlay, PanelView, Region};
use crate::fonts;
use crate::liquid::{self, Blob};
use crate::theme::Theme;

const WIDTH: f32 = 250.0;
const HEIGHT: f32 = 330.0;
const RADIUS: f32 = 30.0;

// the panel reaches this far past the screen's right edge
const EDGE_OVERLAP: f32 = 40.0;

// sits just below the bar so its top-left corner is cleanly rounded inward
const TOP: f32 = 8.0;

// how far past its own width it slides to let go of the edge
const HIDDEN_MARGIN: f32 = 32.0;

const PADDING: Padding = Padding {
    top: 16.0,
    right: EDGE_OVERLAP + 12.0,
    bottom: 16.0,
    left: 16.0,
};

pub const ACTION_COUNT: usize = 5;

const BUTTON_WIDTH: f32 = WIDTH - PADDING.left - PADDING.right;
const BUTTON_HEIGHT: f32 = (HEIGHT - PADDING.top - PADDING.bottom) / ACTION_COUNT as f32;
const BUTTON_RADIUS: f32 = 12.0;

struct Action {
    label: &'static str,
    icon: &'static str,
    command: &'static str,

    // shown in red on hover
    danger: bool,
}

const ACTIONS: [Action; ACTION_COUNT] = [
    Action {
        label: "Shutdown",
        icon: "\u{23fb}",
        command: "systemctl poweroff",
        danger: true,
    },
    Action {
        label: "Lock",
        icon: "\u{f033e}",
        command: "loginctl lock-session",
        danger: false,
    },
    Action {
        label: "Restart",
        icon: "\u{f0709}",
        command: "systemctl reboot",
        danger: false,
    },
    Action {
        label: "Sleep",
        icon: "\u{f04b2}",
        command: "systemctl suspend",
        danger: false,
    },
    Action {
        label: "Logout",
        icon: "\u{f0343}",
        command: "niri msg action quit --skip-confirmation",
        danger: false,
    },
];

// none while fully hidden, anchored to the right screen edge
pub fn view(overlay: &Overlay, theme: &Theme, screen: Region) -> Option<PanelView> {
    let progress = overlay.power_menu.progress.value();

    if !overlay.power_menu.shown && progress <= 0.001 {
        return None;
    }

    let hidden = WIDTH + HIDDEN_MARGIN;

    let x = screen.width - WIDTH + EDGE_OVERLAP + hidden * (1.0 - progress);

    let blob = Blob::new(x, TOP, WIDTH, HEIGHT)
        .radius(RADIUS)
        .child(content(overlay, theme));

    let input = Region {
        x,
        y: TOP,
        width: WIDTH,
        height: HEIGHT,
    };

    // fully out it starts at its resting edge, and its shadow spreads a little past that
    let reach_width = WIDTH - EDGE_OVERLAP + liquid::CONNECTION;

    let reach = Region {
        x: screen.width - reach_width,
        y: 0.0,
        width: reach_width,
        height: TOP + HEIGHT + liquid::CONNECTION,
    };

    Some(PanelView { blob, input, reach })
}

fn content(overlay: &Overlay, theme: &Theme) -> Rectangle {
    let mut buttons: Vec<Box<dyn Widget>> = Vec::new();

    for (index, action) in ACTIONS.iter().enumerate() {
        let fill = overlay.action_fills[index].value();

        let last = index == ACTION_COUNT - 1;

        buttons.push(Box::new(button(index, action, fill, last, theme)));
    }

    Rectangle::new()
        .width(WIDTH)
        .height(HEIGHT)
        .padding(PADDING)
        .on_hover(Overlay::hover_power_menu)
        .child(Column::new(buttons))
}

/*
 * three layers: the hover color filling in from the left, the icon and
 * label, and a thin line under every button but the last
 */
fn button(index: usize, action: &Action, fill: f32, last: bool, theme: &Theme) -> Stack {
    let hovered = fill > 0.5;

    let fill_color = if action.danger {
        theme.danger
    } else {
        theme.accent
    };

    let (icon_color, label_color) = if hovered {
        (theme.on_accent, theme.on_accent)
    } else {
        (theme.text, theme.secondary_text)
    };

    let background = Rectangle::new()
        .width(BUTTON_WIDTH)
        .height(BUTTON_HEIGHT)
        .radius(BUTTON_RADIUS)
        .clip()
        .child(
            Rectangle::new()
                .width(BUTTON_WIDTH * fill)
                .height(BUTTON_HEIGHT)
                .radius(BUTTON_RADIUS)
                .fill(fill_color),
        );

    let icon = Rectangle::new()
        .width(28.0)
        .height(BUTTON_HEIGHT)
        .align_child(Center, Center)
        .child(Text::new(action.icon).size(20.0).font(fonts::NERD).color(icon_color));

    let label = Text::new(action.label)
        .size(14.0)
        .font(fonts::BODY)
        .weight(Weight::Medium)
        .color(label_color);

    let command = action.command;

    let front = Rectangle::new()
        .width(BUTTON_WIDTH)
        .height(BUTTON_HEIGHT)
        .padding(Padding {
            top: 0.0,
            right: 12.0,
            bottom: 0.0,
            left: 12.0,
        })
        .cursor(Pointer)
        .on_hover(move |inside| hover_action(index, inside))
        .on_click(move |_: Button| run(command))
        .align_child(Start, Center)
        .child(Row::new(children![icon, label]).gap(12.0).align(Center));

    let border = Rectangle::new()
        .width(BUTTON_WIDTH - 24.0)
        .height(1.0)
        .fill(if last { Color::TRANSPARENT } else { theme.border });

    let border = Rectangle::new()
        .width(BUTTON_WIDTH)
        .height(BUTTON_HEIGHT)
        .align_child(Center, End)
        .child(border);

    Stack::new(children![background, front, border])
}

// starts the action and closes the menu
fn run(command: &'static str) {
    Overlay::toggle_power_menu(None);

    amane::spawn(command);
}

fn hover_action(index: usize, inside: bool) {
    Overlay::write().action_fills[index].to(if inside { 1.0 } else { 0.0 });
}

pub fn ipc(arguments: &[String]) -> String {
    let command = arguments.first().map(String::as_str).unwrap_or("toggle");
    let monitor_arg = arguments.get(1).map(String::as_str);

    let shown = Overlay::read().power_menu.shown;

    let wanted = match command {
        "show" => true,
        "hide" => false,
        _ => !shown,
    };

    if wanted {
        let mut overlay = Overlay::write();
        overlay.set_active_monitor(monitor_arg);
        overlay.power_menu.show();
    } else {
        Overlay::write().power_menu.hide();
    }

    String::from("ok")
}
