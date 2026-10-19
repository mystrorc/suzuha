use amane::{Color, Full, Layer, LayerWindow, Mask, Monitor, Parent, Rectangle, Zone};

use crate::wallpaper::SCREEN_RADIUS;

/*
 * With an invisible top bar background and square screen corners (SCREEN_RADIUS == 0.0),
 * keep the screen mask unmapped so the compositor doesn't composite a full-screen Top layer.
 */
pub fn view(_monitor: &Monitor) -> LayerWindow {
    let hole = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .radius(SCREEN_RADIUS)
        .fill(Mask);

    LayerWindow::new()
        .width(Full)
        .height(Full)
        .layer(Layer::Top)
        .space(Zone::Respect)
        .namespace("screen-mask")
        .visible(SCREEN_RADIUS > 0.0)
        .click_through()
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(Color::TRANSPARENT)
                .child(hole),
        )
}
