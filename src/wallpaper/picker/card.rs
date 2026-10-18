use amane::{Color, Image, Parent, Pointer, Rectangle, Service};

use super::Picker;

const THUMBNAIL_WIDTH: u32 = super::CARD_WIDTH as u32;
const THUMBNAIL_HEIGHT: u32 = super::CARD_HEIGHT as u32;

const RADIUS: f32 = 16.0;
const EDGE_SHADE: f32 = 0.78;

pub struct Place {
    pub x: f32,
    pub y: f32,

    pub width: f32,
    pub height: f32,

    pub prominence: f32,
    pub offset: i64,
    pub is_center: bool,
}

pub fn view(path: &str, place: Place) -> Rectangle {
    let prominence = place.prominence;

    let scale = 0.4 + prominence * 0.95;
    let opacity = 0.55 + prominence * 0.45;
    let shade = (1.0 - prominence) * EDGE_SHADE;

    let darkness = 1.0 - opacity * (1.0 - shade);
    let shading_alpha = (darkness * 255.0).round() as u8;

    let offset = place.offset;
    let is_center = place.is_center;
    let chosen_path = path.to_string();

    let picture = Image::cover(path).thumbnail(THUMBNAIL_WIDTH, THUMBNAIL_HEIGHT);

    let shading = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .radius(RADIUS)
        .fill(Color::rgba(0, 0, 0, shading_alpha));

    Rectangle::new()
        .width(place.width)
        .height(place.height)
        .radius(RADIUS)
        .fill(picture)
        .scale(scale)
        .translate(place.x - place.width / 2.0, place.y - place.height / 2.0)
        .cursor(Pointer)
        .on_click(move |_| {
            if is_center {
                crate::wallpaper::choose(&chosen_path);
                Picker::write().hide();
            } else {
                Picker::write().slide(offset);
            }
        })
        .child(shading)
}
