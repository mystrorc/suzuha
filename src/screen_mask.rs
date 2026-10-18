use amane::{Color, Full, Layer, LayerWindow, Mask, Monitor, Parent, Rectangle, Zone};

use crate::wallpaper::SCREEN_RADIUS;

/*
 * With an invisible top bar background, the screen mask is kept transparent
 * so no dark corner frames appear below the bar.
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
        .click_through()
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(Color::TRANSPARENT)
                .child(hole),
        )
}
