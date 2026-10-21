use amane::{
    Center, Color, Column, End, Padding, Pointer, Rectangle, Row, Service, Stack, Start, Text,
    Weight, Widget, children,
};

use crate::clock::{self, Clock};
use crate::fonts;
use crate::motion::{self, DEFAULT_SPATIAL};
use crate::overlay::Overlay;
use crate::theme::Theme;

const PADDING: f32 = 14.0;
const TITLE_HEIGHT: f32 = 28.0;
const WEEKDAYS_HEIGHT: f32 = 18.0;
const GAP: f32 = 6.0;
const CELL_GAP: f32 = 2.0;
const ROWS: usize = 6;
const NAVIGATION_SIZE: f32 = 28.0;

const WEEKDAYS: [&str; 7] = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];

const MONTHS: [&str; 12] = [
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December",
];

const PREVIOUS_ICON: &str = "\u{f053}"; // Nerd Font chevron-left
const NEXT_ICON: &str = "\u{f054}";     // Nerd Font chevron-right

struct Day {
    number: i64,
    this_month: bool,
    today: bool,
}

// Two-column layout rendered inside the center pulldown's Calendar tab
pub fn embedded_view(overlay: &Overlay, theme: &Theme, width: f32, height: f32) -> Row {
    let gap = 14.0;
    let col_width = (width - gap) / 2.0;

    let left_card = today_card(overlay, theme, col_width, height);
    let right_card = calendar_card(overlay, theme, col_width, height);

    Row::new(children![left_card, right_card])
        .width(width)
        .height(height)
        .gap(gap)
        .align(Center)
}

// Left card: Date summary, big day badge, live clock, and return-to-today button
fn today_card(overlay: &Overlay, theme: &Theme, width: f32, height: f32) -> Rectangle {
    let clock = Clock::read();
    let today = clock.today();
    let (year, month, day) = today;
    let month_name = MONTHS[(month - 1) as usize];

    // Day of the week
    let date_str = clock.date();
    let weekday = date_str.split(',').next().unwrap_or("Today");

    let weekday_text = Text::new(weekday)
        .size(16.0)
        .font(fonts::BODY)
        .weight(Weight::SemiBold)
        .color(theme.accent);

    let day_number = Text::new(day.to_string())
        .size(64.0)
        .font(fonts::BODY)
        .weight(Weight::Bold)
        .color(theme.text);

    let month_year = Text::new(format!("{month_name} {year}"))
        .size(17.0)
        .font(fonts::BODY)
        .weight(Weight::Medium)
        .color(theme.secondary_text);

    let time_text = Text::new(clock.time(false))
        .size(24.0)
        .font(fonts::BODY)
        .weight(Weight::SemiBold)
        .color(theme.text);

    let mut elements: Vec<Box<dyn Widget>> = vec![
        Box::new(weekday_text),
        Box::new(day_number),
        Box::new(month_year),
        Box::new(Rectangle::new().width(width - PADDING * 4.0).height(1.0).fill(theme.border)),
        Box::new(time_text),
    ];

    if overlay.calendar_month != 0 {
        let return_btn = Rectangle::new()
            .width(130.0)
            .height(28.0)
            .radius(8.0)
            .fill(theme.selected_surface)
            .cursor(Pointer)
            .on_click(|_| Overlay::write().calendar_month = 0)
            .align_child(Center, Center)
            .child(
                Text::new("Return to Today")
                    .size(11.0)
                    .font(fonts::BODY)
                    .weight(Weight::Medium)
                    .color(theme.accent),
            );

        elements.push(Box::new(return_btn));
    }

    let content = Column::new(elements)
        .gap(8.0)
        .align(Center);

    Rectangle::new()
        .width(width)
        .height(height)
        .radius(20.0)
        .fill(theme.surface)
        .align_child(Center, Center)
        .child(content)
}

// Right card: Interactive month calendar grid with navigation
fn calendar_card(overlay: &Overlay, theme: &Theme, width: f32, height: f32) -> Rectangle {
    let inner_width = width - PADDING * 2.0;
    let inner_height = height - PADDING * 2.0;

    let chosen = overlay.calendar_month as f32;
    let shown = motion::follow("calendar-month", chosen, DEFAULT_SPATIAL);
    let today = Clock::read().today();

    let mut pages: Vec<Box<dyn Widget>> = Vec::new();
    let nearest = shown.floor() as i64;

    for offset in [nearest, nearest + 1] {
        let distance = offset as f32 - shown;
        if distance.abs() >= 1.0 {
            continue;
        }

        let page = month(overlay, theme, today, offset, inner_width, inner_height)
            .translate(distance * inner_width, 0.0);

        pages.push(Box::new(page));
    }

    let mut viewport = Rectangle::new()
        .width(inner_width)
        .height(inner_height)
        .child(Stack::new(pages));

    if shown != chosen {
        viewport = viewport.clip();
    }

    let navigation = Row::new(children![
        nav_button(overlay, theme, "prev", PREVIOUS_ICON, -1),
        nav_button(overlay, theme, "next", NEXT_ICON, 1),
    ])
    .gap(6.0);

    let nav_bar = Rectangle::new()
        .width(inner_width)
        .height(NAVIGATION_SIZE)
        .align_child(End, Start)
        .child(navigation);

    Rectangle::new()
        .width(width)
        .height(height)
        .radius(20.0)
        .fill(theme.surface)
        .padding(Padding {
            top: PADDING,
            right: PADDING,
            bottom: PADDING,
            left: PADDING,
        })
        .child(Stack::new(children![viewport, nav_bar]))
}

fn month(
    overlay: &Overlay,
    theme: &Theme,
    today: (i64, i64, i64),
    offset: i64,
    width: f32,
    height: f32,
) -> Rectangle {
    let (today_year, today_month, _) = today;
    let index = today_year * 12 + (today_month - 1) + offset;
    let year = index.div_euclid(12);
    let month = index.rem_euclid(12) + 1;

    let title_text = Text::new(format!("{} {year}", MONTHS[(month - 1) as usize]))
        .size(15.0)
        .font(fonts::BODY)
        .weight(Weight::SemiBold)
        .color(theme.text);

    let mut title_bar = Rectangle::new()
        .width(width - NAVIGATION_SIZE * 2.0 - 12.0)
        .height(TITLE_HEIGHT)
        .align_child(Start, Center)
        .child(title_text);

    if offset != 0 {
        title_bar = title_bar
            .cursor(Pointer)
            .on_click(|_| Overlay::write().calendar_month = 0);
    }

    let grid_height = height - TITLE_HEIGHT - WEEKDAYS_HEIGHT - GAP * 2.0;

    let column = Column::new(children![
        title_bar,
        weekdays(theme, width),
        grid(overlay, theme, &days(today, year, month), offset, width, grid_height),
    ])
    .gap(GAP);

    Rectangle::new().width(width).height(height).child(column)
}

fn weekdays(theme: &Theme, width: f32) -> Row {
    let cell_width = width / 7.0;
    let mut names: Vec<Box<dyn Widget>> = Vec::with_capacity(7);

    for name in WEEKDAYS {
        let text = Text::new(name)
            .size(9.0)
            .font(fonts::BODY)
            .color(theme.muted_text);

        let cell = Rectangle::new()
            .width(cell_width)
            .height(WEEKDAYS_HEIGHT)
            .align_child(Center, Center)
            .child(text);

        names.push(Box::new(cell));
    }

    Row::new(names)
}

fn grid(
    overlay: &Overlay,
    theme: &Theme,
    days: &[Day],
    offset: i64,
    width: f32,
    height: f32,
) -> Column {
    let cell_width = (width - CELL_GAP * 6.0) / 7.0;
    let cell_height = (height - CELL_GAP * (ROWS - 1) as f32) / ROWS as f32;

    let mut rows: Vec<Box<dyn Widget>> = Vec::with_capacity(ROWS);

    for week in 0..ROWS {
        let mut cells: Vec<Box<dyn Widget>> = Vec::with_capacity(7);

        for weekday in 0..7 {
            let index = week * 7 + weekday;
            let hover_name = format!("cal:{offset}:{index}");

            let cell = cell(overlay, theme, &days[index], hover_name, cell_width, cell_height);
            cells.push(Box::new(cell));
        }

        rows.push(Box::new(Row::new(cells).gap(CELL_GAP)));
    }

    Column::new(rows).gap(CELL_GAP)
}

fn cell(
    overlay: &Overlay,
    theme: &Theme,
    day: &Day,
    hover_name: String,
    width: f32,
    height: f32,
) -> Rectangle {
    let is_hovered = overlay.hovered.as_deref() == Some(&hover_name);

    let fill = if day.today {
        theme.accent
    } else if day.this_month && is_hovered {
        theme.selected_surface
    } else {
        Color::TRANSPARENT
    };

    let color = if day.today {
        theme.on_accent
    } else if day.this_month {
        theme.text
    } else {
        theme.border
    };

    let weight = if day.today {
        Weight::Bold
    } else {
        Weight::Light
    };

    let number = Text::new(day.number.to_string())
        .size(10.0)
        .font(fonts::BODY)
        .weight(weight)
        .color(color);

    Rectangle::new()
        .width(width)
        .height(height)
        .radius(8.0)
        .fill(fill)
        .on_hover(move |inside| {
            let mut o = Overlay::write();
            if inside {
                o.hovered = Some(hover_name.clone());
            } else if o.hovered.as_ref() == Some(&hover_name) {
                o.hovered = None;
            }
        })
        .align_child(Center, Center)
        .child(number)
}

fn days(today: (i64, i64, i64), year: i64, month: i64) -> Vec<Day> {
    let first = clock::days_since_1970(year, month, 1);
    let weekday = (first + 4).rem_euclid(7);
    let start = first - weekday;

    let (today_year, today_month, today_day) = today;
    let today_days = clock::days_since_1970(today_year, today_month, today_day);

    let mut days = Vec::with_capacity(ROWS * 7);

    for index in 0..(ROWS * 7) as i64 {
        let date = start + index;
        let (_, date_month, date_day) = clock::civil_date(date);

        days.push(Day {
            number: date_day,
            this_month: date_month == month,
            today: date == today_days,
        });
    }

    days
}

fn nav_button(
    overlay: &Overlay,
    theme: &Theme,
    name: &str,
    icon: &str,
    step: i64,
) -> Rectangle {
    let hover_name = format!("cal_nav:{name}");
    let is_hovered = overlay.hovered.as_deref() == Some(&hover_name);

    let fill = if is_hovered {
        theme.border
    } else {
        theme.selected_surface
    };

    let glyph = Text::new(icon)
        .size(11.0)
        .font(fonts::NERD)
        .tight()
        .color(theme.text);

    let hover_key = hover_name.clone();

    Rectangle::new()
        .width(NAVIGATION_SIZE)
        .height(NAVIGATION_SIZE)
        .radius(8.0)
        .fill(fill)
        .cursor(Pointer)
        .on_click(move |_| Overlay::write().calendar_month += step)
        .on_hover(move |inside| {
            let mut o = Overlay::write();
            if inside {
                o.hovered = Some(hover_key.clone());
            } else if o.hovered.as_ref() == Some(&hover_key) {
                o.hovered = None;
            }
        })
        .align_child(Center, Center)
        .child(glyph)
}
