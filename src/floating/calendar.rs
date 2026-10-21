use amane::{
    Center, Color, Column, Rectangle, Row, Service, Start, Text, Weight, Widget, children,
};

use crate::clock::{self, Clock};
use crate::fonts;
use crate::theme::Theme;

use super::card;

pub const WIDTH: f32 = 220.0;
pub const HEIGHT: f32 = 190.0;

const PADDING: f32 = 18.0;

const ROWS: usize = 6;
const CELL_GAP: f32 = 2.0;

const WEEKDAYS: [&str; 7] = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];

const MONTHS: [&str; 12] = [
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December",
];

struct Day {
    number: i64,
    this_month: bool,
    today: bool,
}

pub fn view(theme: &Theme) -> Rectangle {
    let clock = Clock::read();
    let today = clock.today();
    let (today_year, today_month, _) = today;

    let inner_width = WIDTH - PADDING * 2.0;

    let title = Text::new(format!("{} {today_year}", MONTHS[(today_month - 1) as usize]))
        .size(14.0)
        .font(fonts::BODY)
        .weight(Weight::SemiBold)
        .color(theme.text);

    let title_bar = Rectangle::new()
        .width(inner_width)
        .height(20.0)
        .align_child(Start, Center)
        .child(title);

    let weekdays_row = weekdays(theme, inner_width);

    let grid_height = HEIGHT - PADDING * 2.0 - 20.0 - 14.0 - 10.0;
    let days_list = days(today);
    let days_grid = grid(theme, &days_list, inner_width, grid_height);

    let content = Column::new(children![
        title_bar,
        weekdays_row,
        days_grid,
    ])
    .gap(5.0);

    card::background(theme, WIDTH, HEIGHT).child(content)
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
            .height(14.0)
            .align_child(Center, Center)
            .child(text);

        names.push(Box::new(cell));
    }

    Row::new(names)
}

fn grid(theme: &Theme, days: &[Day], width: f32, height: f32) -> Column {
    let cell_width = (width - CELL_GAP * 6.0) / 7.0;
    let cell_height = (height - CELL_GAP * (ROWS - 1) as f32) / ROWS as f32;

    let mut rows: Vec<Box<dyn Widget>> = Vec::with_capacity(ROWS);

    for week in 0..ROWS {
        let mut cells: Vec<Box<dyn Widget>> = Vec::with_capacity(7);

        for weekday in 0..7 {
            let index = week * 7 + weekday;
            let cell = cell(theme, &days[index], cell_width, cell_height);
            cells.push(Box::new(cell));
        }

        rows.push(Box::new(Row::new(cells).gap(CELL_GAP)));
    }

    Column::new(rows).gap(CELL_GAP)
}

fn cell(theme: &Theme, day: &Day, width: f32, height: f32) -> Rectangle {
    let fill = if day.today {
        theme.accent
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
        .radius(6.0)
        .fill(fill)
        .align_child(Center, Center)
        .child(number)
}

fn days(today: (i64, i64, i64)) -> Vec<Day> {
    let (today_year, today_month, today_day) = today;
    let first = clock::days_since_1970(today_year, today_month, 1);

    // 1970-01-01 was a thursday, 4 days after a sunday
    let weekday = (first + 4).rem_euclid(7);

    let start = first - weekday;
    let today_days = clock::days_since_1970(today_year, today_month, today_day);

    let mut days = Vec::with_capacity(ROWS * 7);

    for index in 0..(ROWS * 7) as i64 {
        let date = start + index;
        let (_, date_month, date_day) = clock::civil_date(date);

        days.push(Day {
            number: date_day,
            this_month: date_month == today_month,
            today: date == today_days,
        });
    }

    days
}
