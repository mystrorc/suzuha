mod blurred;
pub mod picker;
mod reveal;
mod shuffle;
mod state;

use std::sync::atomic::Ordering;

use amane::{
    Color, Full, Image, Layer, LayerWindow, Monitor, Parent, Rectangle, Service, Stack,
    Zone, children,
};

use crate::lock_screen::{self, Curtain};
use crate::settings::Settings;
use crate::theme;

pub use blurred::blurred;
pub use shuffle::Shuffle;
pub use state::{Wallpaper, choose, is_transitioning};

// Square screen corners (no dark rounded corners at the top-left/top-right of the screen)
const FRAME_RADIUS: f32 = 0.0;

pub const SCREEN_RADIUS: f32 = 0.0;

// under everything, taking no clicks; niri also shows it behind the overview
pub fn view(monitor: &Monitor) -> LayerWindow {
    let theme = theme::current();

    let wallpaper = Wallpaper::read();

    let curtain = Curtain::read();

    // square once the screen mask fades for a lock, matching the lock screen
    let radius = FRAME_RADIUS * curtain.items.value();

    let width = monitor.width as f32;
    let height = monitor.height as f32;

    let backdrop = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(theme.background)
        .opacity(wallpaper.backdrop.value());

    if Image::loaded(&wallpaper.shown) {
        state::DRAWN.store(true, Ordering::Relaxed);
    }

    let below = (1.0 - wallpaper.rise.value()) * height;

    let mut frame = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .radius(radius)
        .fill(Image::cover(&wallpaper.shown))
        .translate(0.0, below);

    if let Some(incoming) = &wallpaper.incoming {
        let center = (wallpaper.center.0 * width, wallpaper.center.1 * height);

        let progress = wallpaper.reveal.value();

        // a fade lays the whole new picture on top and brings it in
        if Settings::read().text("wallpaper_transition") == "fade" {
            let picture = Rectangle::new()
                .width(Parent)
                .height(Parent)
                .radius(radius)
                .fill(Image::cover(incoming))
                .opacity(progress);

            frame = frame.child(picture);
        } else {
            frame = frame.child(reveal::view(incoming, center, progress, width, height));
        }
    }

    let dim = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(Color::rgba(0, 0, 0, lock_screen::DIM))
        .opacity(curtain.dark.value());

    LayerWindow::new()
        .width(Full)
        .height(Full)
        .layer(Layer::Background)
        .space(Zone::Ignore)
        .namespace("wallpaper")
        .click_through()
        .child(Stack::new(children![backdrop, frame, dim]).width(Parent).height(Parent))
}
