mod center;
pub mod home_assistant;
mod motion;
mod pill;
mod reveal;
mod star;
mod ring;
mod system;
mod timer;
mod workspaces;

use amane::{
    Align, Color, Full, Layer, LayerWindow, Monitor, Parent, Rectangle, Row, Service, Vertical,
    Zone, children,
};

use crate::lock_screen::Curtain;
use crate::motion::{self as shell_motion, DEFAULT_SPATIAL};
use crate::settings::Settings;
use crate::theme;

use reveal::Reveal;

pub const ITEM_HEIGHT: f32 = 32.0;

// Width reserved on the left (Power Menu) and right (System Tray) so the middle section stays centered
const SIDE_WIDTH: f32 = 140.0;

// what is left of an auto-hidden bar, for the pointer to find
const TRIGGER: f32 = 2.0;

// the bar's height from the settings
pub fn height() -> f32 {
    Settings::read().number("bar_height")
}

// the room the bar keeps free for itself, none while it hides
pub fn reserved() -> f32 {
    if Settings::read().flag("bar_auto_hide") {
        return 0.0;
    }

    height()
}

// the room kept free above the rest of the screen
pub fn reserved_top() -> f32 {
    if Settings::read().text("bar_position") == "bottom" {
        return 0.0;
    }

    reserved()
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    let theme = theme::current();

    let settings = Settings::read();

    let height = settings.number("bar_height");
    let bottom = settings.text("bar_position") == "bottom";
    let hides = settings.flag("bar_auto_hide");

    drop(settings);

    // how far the bar is out, always all the way unless it hides
    let target = if !hides || Reveal::read().inside { 1.0 } else { 0.0 };

    let shown = shell_motion::follow(&format!("bar:{}", monitor.name), target, DEFAULT_SPATIAL);

    let window_height = (height * shown).max(TRIGGER);

    let total_width = monitor.width as f32;
    let side_width = SIDE_WIDTH.min(total_width * 0.2);
    let center_width = (total_width - side_width * 2.0).max(0.0);

    let align = if bottom { Align::End } else { Align::Start };

    let sections = Row::new(children![
        workspaces::view(monitor, &theme, side_width, align),
        center::view(monitor, &theme, center_width, align),
        system::view(monitor, &theme, side_width, align),
    ])
    .align(align);

    let content = Rectangle::new()
        .width(Parent)
        .height(height)
        .child(sections);

    // a hiding bar slides toward its edge, the part past the window is clipped
    let hidden = height - window_height;

    let slide = if bottom { 0.0 } else { -hidden };

    let edge = if bottom {
        Vertical::Bottom
    } else {
        Vertical::Top
    };

    let space = if hides { Zone::Ignore } else { Zone::Reserve };

    // Invisible top bar background: only the pills and icons are drawn
    LayerWindow::new()
        .width(Full)
        .height(window_height)
        .anchor_vertical(edge)
        .layer(Layer::Top)
        .space(space)
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(Color::TRANSPARENT)
                .opacity(Curtain::read().items.value())
                .clip()
                .on_hover(reveal::hover)
                .child(Rectangle::new().width(Parent).height(height).translate(0.0, slide).child(content)),
        )
}
