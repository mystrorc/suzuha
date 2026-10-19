use amane::{Center, Column, Padding, Pointer, Rectangle, Row, Service, Stack, Start, Text, Weight, Widget, children};

use super::{PAGES, Page, Shown, hover, hovered};
use crate::fonts;
use crate::motion::{self, FAST_SPATIAL};
use crate::theme::Theme;

pub const WIDTH: f32 = 218.0;

const ITEM_HEIGHT: f32 = 34.0;
const ITEM_GAP: f32 = 2.0;

const TOP: f32 = 10.0;
const SIDE: f32 = 10.0;

// the list of pages, with the selection sliding up and down behind it
pub fn view(theme: &Theme, shown: Page, height: f32) -> Rectangle {
    let item_width = WIDTH - SIDE * 2.0;

    let mut position = 0;
    let mut items: Vec<Box<dyn Widget>> = Vec::new();

    for (index, (page, icon, label, _)) in PAGES.iter().enumerate() {
        let current = *page == shown;

        if current {
            position = index;
        }

        items.push(Box::new(item(theme, *page, icon, label, current, item_width)));
    }

    let target = TOP + position as f32 * (ITEM_HEIGHT + ITEM_GAP);

    let y = motion::follow("settings-navigation", target, FAST_SPATIAL);

    let selection = Rectangle::new()
        .width(item_width)
        .height(ITEM_HEIGHT)
        .radius(10.0)
        .fill(theme.selected_surface)
        .translate(SIDE, y);

    let list = Rectangle::new()
        .width(WIDTH)
        .height(height)
        .padding(Padding {
            top: TOP,
            right: SIDE,
            bottom: 0.0,
            left: SIDE,
        })
        .child(Column::new(items).gap(ITEM_GAP));

    Rectangle::new()
        .width(WIDTH)
        .height(height)
        .radius(20.0)
        .fill(theme.surface)
        .clip()
        .child(Stack::new(children![selection, list]).width(WIDTH).height(height))
}

fn item(theme: &Theme, page: Page, icon: &str, label: &str, current: bool, width: f32) -> Rectangle {
    let hover_name = format!("navigation:{label}");

    // the hover shows on every page but the current one, which the selection already marks
    let fill = if hovered(&hover_name) && !current {
        theme.hover_surface
    } else {
        amane::Color::TRANSPARENT
    };

    let (icon_color, text_color, weight) = if current {
        (theme.accent, theme.text, Weight::SemiBold)
    } else {
        (theme.secondary_text, theme.secondary_text, Weight::Regular)
    };

    let icon = Rectangle::new()
        .width(22.0)
        .height(22.0)
        .align_child(Center, Center)
        .child(Text::new(icon).size(14.0).font(fonts::NERD).tight().color(icon_color));

    let label = Text::new(label)
        .size(12.5)
        .font(fonts::BODY)
        .weight(weight)
        .color(text_color)
        .elide();

    Rectangle::new()
        .width(width)
        .height(ITEM_HEIGHT)
        .radius(10.0)
        .fill(fill)
        .cursor(Pointer)
        .on_hover(move |inside| hover(hover_name.clone(), inside))
        .on_click(move |_| select(page))
        .align_child(Start, Center)
        .padding(Padding {
            top: 0.0,
            right: 10.0,
            bottom: 0.0,
            left: 10.0,
        })
        .child(Row::new(children![icon, label]).gap(9.0).align(Center))
}

fn select(page: Page) {
    Shown::write().page = page;
}
