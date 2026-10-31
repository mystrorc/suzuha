mod calendar;
mod card;
mod memory;
mod placement;
pub mod sensors;
pub mod weather;
pub mod windows;

use std::collections::HashMap;
use std::time::SystemTime;

use amane::{
    Align, Column, Cpu, End, Full, Layer, LayerWindow, Monitor, Parent, Rectangle, Row, Service,
    SpaceBetween, Stack, Start, Text, Weight, Widget, Workspaces, Zone, children,
};

use crate::bar;
use crate::clock::Clock;
use crate::fonts;
use crate::lock_screen::Curtain;
use crate::motion::{self, DEFAULT_SPATIAL};
use crate::settings::Settings;
use crate::theme::{self, Theme};
use crate::wallpaper::{self, Wallpaper};

use card::Reading;
use placement::{Name, Placement, Request};
use sensors::Sensors;
pub use weather::Weather;

const MARGIN: f32 = 36.0;
const GAP: f32 = 16.0;

const WEATHER_WIDTH: f32 = 220.0;
const WEATHER_HEIGHT: f32 = 160.0;

const CLOCK_WIDTH: f32 = 300.0;
const CLOCK_HEIGHT: f32 = 132.0;

// how long a card takes to glide to its spot on a new wallpaper
const MOVE: u64 = 650;

const THERMOMETER: &str = "\u{f050f}";
const PROCESSOR: &str = "\u{f035b}";
const SUN: &str = "\u{f0599}";
const DROP: &str = "\u{f058c}";
const LEAF: &str = "\u{f032a}";

/*
 * cards on the desktop, under every window, placed where the wallpaper is
 * calm enough to read them; they only show on a workspace with no windows,
 * where the desktop is actually seen
 */
pub fn view(monitor: &Monitor) -> LayerWindow {
    let width = monitor.width as f32;
    let height = monitor.height as f32 - bar::reserved();

    let empty = Workspaces::read()
        .list()
        .iter()
        .find(|workspace| workspace.active() && workspace.output() == Some(&monitor.name))
        .is_some_and(|workspace| workspace.windows() == 0);

    let has_half_desktop = windows::DesktopWindows::read().occupied_half(&monitor.name).is_some();

    let settings = Settings::read();

    let wanted = match settings.text("floating_visibility") {
        "always" => true,
        "hidden" => false,
        _ => empty || has_half_desktop,
    };

    let scale = settings.number("floating_scale");
    let opacity = settings.number("floating_opacity");
    let show_clock = settings.flag("widget_clock");

    drop(settings);

    // at startup the cards wait for the wallpaper to finish rising
    let risen = Wallpaper::read().risen;

    // the cards fade in and out, so the window stays until they are gone
    let target = if wanted && risen { 1.0 } else { 0.0 };

    let shown = motion::appear(&format!("floating:{}", monitor.name), target, DEFAULT_SPATIAL);

    if !wanted && shown <= 0.001 {
        return LayerWindow::new()
            .width(Full)
            .height(Full)
            .layer(Layer::Bottom)
            .namespace("floating-widgets")
            .visible(false);
    }

    let theme = theme::current();
    let target_theme = theme::target();

    let mut cards = cards(&theme);

    let mut names: Vec<Name> = cards.iter().map(|(name, _)| *name).collect();

    names.push(Name::Clock);

    // nothing shows until the cards know their spots, so they fade in where they stay
    let Some(spots) = spots(&target_theme, &monitor.name, &names, width, height, scale) else {
        return LayerWindow::new()
            .width(Full)
            .height(Full)
            .layer(Layer::Bottom)
            .namespace("floating-widgets")
            .visible(false);
    };

    // the clock's text leans toward the screen edge it sits nearest, so it never floats off it
    let clock_center = spots[&Name::Clock].0 + CLOCK_WIDTH * scale / 2.0;

    if show_clock {
        cards.push((Name::Clock, clock(&theme, clock_center < width / 2.0)));
    }

    let mut layers: Vec<Box<dyn Widget>> = Vec::new();

    for (name, card) in cards {
        let (x, y) = spots[&name];

        // a card grows around its center, so it is moved by half its growth to keep its corner on the spot
        let (card_width, card_height) = size(name, 1.0);

        let x = x + card_width * (scale - 1.0) / 2.0;
        let y = y + card_height * (scale - 1.0) / 2.0;

        // each card glides on its own, named by monitor so screens never share a glide
        let key = format!("floating:{}:{name:?}", monitor.name);

        let x = motion::follow(&format!("{key}:x"), x, MOVE);
        let y = motion::follow(&format!("{key}:y"), y, MOVE);

        // the clock floats without a card, so there is no glass behind it
        if name != Name::Clock {
            let left = x - card_width * (scale - 1.0) / 2.0;
            let top = y - card_height * (scale - 1.0) / 2.0;

            layers.push(Box::new(glass(monitor, left, top, name, scale)));
        }

        layers.push(Box::new(card.scale(scale).translate(x, y)));
    }

    LayerWindow::new()
        .width(Full)
        .height(Full)
        .layer(Layer::Bottom)
        .space(Zone::Respect)
        .namespace("floating-widgets")
        .visible(wanted || shown > 0.001)
        .click_through()
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .opacity(shown * opacity * Curtain::read().items.value())
                .child(Stack::new(layers).width(Parent).height(Parent)),
        )
}

/*
 * the blurred wallpaper cut to a card's shape, lined up with the real one;
 * the window starts below a top bar, so the wallpaper is moved up by it
 */
fn glass(monitor: &Monitor, left: f32, top: f32, name: Name, scale: f32) -> Rectangle {
    let (width, height) = size(name, scale);

    let screen_width = monitor.width as f32;
    let screen_height = monitor.height as f32;

    let wallpaper = Rectangle::new()
        .width(screen_width)
        .height(screen_height)
        .fill(wallpaper::blurred(screen_width, screen_height))
        .translate(-left, -top - bar::reserved_top());

    Rectangle::new()
        .width(width)
        .height(height)
        .radius(card::RADIUS * scale)
        .clip()
        .child(wallpaper)
        .translate(left, top)
}

// the room a card takes at the given scale
fn size(name: Name, scale: f32) -> (f32, f32) {
    let (width, height) = match name {
        Name::Clock => (CLOCK_WIDTH, CLOCK_HEIGHT),
        Name::Weather => (WEATHER_WIDTH, WEATHER_HEIGHT),
        Name::Calendar => (calendar::WIDTH, calendar::HEIGHT),
        _ => (card::WIDTH, card::HEIGHT),
    };

    (width * scale, height * scale)
}

/*
 * where every card goes on this screen: the calmest spots on the wallpaper,
 * the remembered ones until it has been read, fixed corners when it can't
 * be or nothing fits, and none while there is nothing to go on yet.
 * If an app is open half screen, widgets dynamically adjust to the free side.
 */
fn spots(
    theme: &Theme,
    monitor: &str,
    names: &[Name],
    width: f32,
    height: f32,
    scale: f32,
) -> Option<HashMap<Name, (f32, f32)>> {
    // If an app is open half screen on this monitor, adjust widgets to the other side
    if let Some(occupied) = windows::DesktopWindows::read().occupied_half(monitor) {
        let free_side = match occupied {
            windows::HalfScreen::Left => windows::HalfScreen::Right,
            windows::HalfScreen::Right => windows::HalfScreen::Left,
        };
        return Some(half_screen_spots(names, width, height, scale, free_side));
    }

    let placement = Placement::read();

    let Some(analysis) = placement.analysis() else {
        if let Some(spots) = memory::recall(monitor, names) {
            return Some(spots);
        }

        if placement.settled() {
            return Some(corners(names, width, height, scale));
        }

        return None;
    };

    let sized = |group: &[Name]| -> Vec<(Name, f32, f32)> {
        let mut sized = Vec::new();

        for name in names.iter().filter(|name| group.contains(name)) {
            let (card_width, card_height) = size(*name, scale);

            sized.push((*name, card_width, card_height));
        }

        sized
    };

    let text = theme.text;

    let text_luminance = (f32::from(text.red()) * 0.2126
        + f32::from(text.green()) * 0.7152
        + f32::from(text.blue()) * 0.0722)
        / 255.0;

    let request = Request {
        screen: (width, height),
        usable: (MARGIN, MARGIN, width - MARGIN * 2.0, height - MARGIN * 2.0),
        clock: size(Name::Clock, scale),
        primary: sized(&[Name::Weather, Name::Calendar]),
        resources: sized(&[Name::CpuTemperature, Name::CpuUsage, Name::GpuTemperature]),
        environment: sized(&[Name::UvIndex, Name::Humidity, Name::AirQuality]),
        text_luminance,
        analysis,
        crop: placement::crop(analysis, width, height),
    };

    let spots = placement::arrange(&request).unwrap_or_else(|| corners(names, width, height, scale));

    memory::remember(monitor, &spots);

    Some(spots)
}

// Arranges widgets into the free half of the screen so they don't sit under half-screen apps
fn half_screen_spots(
    names: &[Name],
    width: f32,
    height: f32,
    scale: f32,
    free_side: windows::HalfScreen,
) -> HashMap<Name, (f32, f32)> {
    let mut spots = HashMap::new();

    let (min_x, max_x) = match free_side {
        windows::HalfScreen::Left => (MARGIN, (width / 2.0) - MARGIN),
        windows::HalfScreen::Right => ((width / 2.0) + MARGIN, width - MARGIN),
    };

    let col1_x = min_x;
    let mut col1_y = MARGIN;

    // Place primary cards (Weather, Calendar) in column 1 (inner/left of the free half)
    for name in [Name::Weather, Name::Calendar] {
        if names.contains(&name) {
            let (_, card_h) = size(name, scale);
            spots.insert(name, (col1_x, col1_y));
            col1_y += card_h + GAP;
        }
    }

    // Place readings (CPU, GPU, UV, Humidity, AQI) in column 2 (outer/right of the free half)
    let readings = [
        Name::CpuTemperature,
        Name::CpuUsage,
        Name::GpuTemperature,
        Name::UvIndex,
        Name::Humidity,
        Name::AirQuality,
    ];

    let reading_w = card::WIDTH * scale;
    let col2_x = max_x - reading_w;
    let mut col2_y = MARGIN;

    let (clock_w, clock_h) = size(Name::Clock, scale);
    let clock_y = height - MARGIN - clock_h;

    for name in readings {
        if names.contains(&name) {
            let (_, card_h) = size(name, scale);
            if col2_y + card_h > clock_y - GAP && col1_y + card_h <= height - MARGIN {
                spots.insert(name, (col1_x, col1_y));
                col1_y += card_h + GAP;
            } else {
                spots.insert(name, (col2_x, col2_y));
                col2_y += card_h + GAP;
            }
        }
    }

    // Place the digital clock at the bottom of column 2 (aligned to max_x)
    if names.contains(&Name::Clock) {
        let clock_x = max_x - clock_w;
        spots.insert(Name::Clock, (clock_x, clock_y));
    }

    spots
}

// readings top left, weather top right, the air bottom left and the clock bottom right
fn corners(names: &[Name], width: f32, height: f32, scale: f32) -> HashMap<Name, (f32, f32)> {
    let mut spots = HashMap::new();

    let mut top_left = MARGIN;
    let mut bottom_left = MARGIN;

    for name in names {
        let (card_width, card_height) = size(*name, scale);

        let spot = match name {
            Name::Clock => (width - MARGIN - card_width, height - MARGIN - card_height),
            Name::Weather => (width - MARGIN - card_width, MARGIN),

            Name::Calendar => {
                bottom_left += card_width + GAP;

                (bottom_left - card_width - GAP, height - MARGIN - card_height)
            }

            Name::CpuTemperature | Name::CpuUsage | Name::GpuTemperature => {
                top_left += card_width + GAP;

                (top_left - card_width - GAP, MARGIN)
            }

            Name::UvIndex | Name::Humidity | Name::AirQuality => {
                bottom_left += card_width + GAP;

                (bottom_left - card_width - GAP, height - MARGIN - card_height)
            }
        };

        spots.insert(*name, spot);
    }

    spots
}

// every card but the clock, which waits to know its side; each named so it can be placed on its own
fn cards(theme: &Theme) -> Vec<(Name, Rectangle)> {
    let sensors = Sensors::read();
    let weather = Weather::read();

    let mut cards = vec![
        (Name::Weather, weather_card(theme, &weather)),
        (Name::Calendar, calendar::view(theme)),
    ];

    if let Some(cpu) = sensors.cpu {
        cards.push((Name::CpuTemperature, temperature(theme, "CPU TEMPERATURE", cpu)));
    }

    let load = Cpu::read().percent();

    cards.push((
        Name::CpuUsage,
        card::view(
            theme,
            Reading {
                label: "CPU USAGE",
                icon: PROCESSOR,
                value: format!("{load}%"),
                amount: f32::from(load) / 100.0,
                color: theme.accent,
            },
        ),
    ));

    if let Some(gpu) = sensors.gpu {
        cards.push((Name::GpuTemperature, temperature(theme, "GPU TEMPERATURE", gpu)));
    }

    let shown = |value: Option<f32>, digits: usize| match value {
        Some(value) => format!("{value:.digits$}"),
        None => String::from("–"),
    };

    cards.push((
        Name::UvIndex,
        card::view(
            theme,
            Reading {
                label: "UV INDEX",
                icon: SUN,
                value: shown(weather.uv_index, 1),
                amount: weather.uv_index.unwrap_or(0.0) / 11.0,
                color: theme.danger,
            },
        ),
    ));

    cards.push((
        Name::Humidity,
        card::view(
            theme,
            Reading {
                label: "HUMIDITY",
                icon: DROP,
                value: match weather.humidity {
                    Some(humidity) => format!("{humidity:.0}%"),
                    None => String::from("–"),
                },
                amount: weather.humidity.unwrap_or(0.0) / 100.0,
                color: theme.accent,
            },
        ),
    ));

    cards.push((
        Name::AirQuality,
        card::view(
            theme,
            Reading {
                label: "AQI",
                icon: LEAF,
                value: shown(weather.air_quality, 0),
                amount: weather.air_quality.unwrap_or(0.0) / 300.0,
                color: theme.accent,
            },
        ),
    ));

    cards.retain(|(name, _)| turned_on(*name));

    cards
}

// whether the settings show this card
fn turned_on(name: Name) -> bool {
    let key = match name {
        Name::Clock => "widget_clock",
        Name::Weather => "widget_weather",
        Name::Calendar => "widget_calendar",
        Name::CpuTemperature => "widget_cpu_temperature",
        Name::CpuUsage => "widget_cpu_usage",
        Name::GpuTemperature => "widget_gpu_temperature",
        Name::UvIndex => "widget_uv",
        Name::Humidity => "widget_humidity",
        Name::AirQuality => "widget_air_quality",
    };

    Settings::read().flag(key)
}

fn temperature(theme: &Theme, label: &str, degrees: u32) -> Rectangle {
    card::view(
        theme,
        Reading {
            label,
            icon: THERMOMETER,
            value: format!("{degrees}°C"),
            amount: degrees as f32 / 100.0,
            color: theme.accent,
        },
    )
}

fn weather_card(theme: &Theme, weather: &Weather) -> Rectangle {

    let degrees = |value: Option<f32>| match value {
        Some(value) => format!("{value:.0}°"),
        None => String::from("–"),
    };

    let temperature = Text::new(degrees(weather.temperature))
        .size(44.0)
        .font(fonts::BODY)
        .weight(Weight::Bold)
        .color(theme.text);

    let condition = Text::new(weather.condition())
        .size(13.0)
        .font(fonts::BODY)
        .weight(Weight::Medium)
        .color(theme.text);

    let range = Text::new(format!("H {}  L {}", degrees(weather.high), degrees(weather.low)))
        .size(12.0)
        .font(fonts::BODY)
        .color(theme.secondary_text);

    let icon = Text::new(weather.icon())
        .size(40.0)
        .font(fonts::NERD)
        .tight()
        .color(theme.accent);

    let bottom = Row::new(children![
        Column::new(children![condition, range]).gap(2.0),
        icon,
    ])
    .width(WEATHER_WIDTH - 36.0)
    .justify(SpaceBetween)
    .align(End);

    card::background(theme, WEATHER_WIDTH, WEATHER_HEIGHT).child(
        Column::new(children![temperature, bottom])
            .height(WEATHER_HEIGHT - 36.0)
            .justify(SpaceBetween),
    )
}

// "14:36" over the date, straight on the desktop without a card, lined up on the left or right
fn clock(theme: &Theme, on_left: bool) -> Rectangle {
    let clock = Clock::read();

    let time = Text::new(clock.hours_minutes(SystemTime::now()))
        .size(72.0)
        .font(fonts::BODY)
        .weight(Weight::Bold)
        .color(theme.text);

    let date = Text::new(clock.date())
        .size(16.0)
        .font(fonts::BODY)
        .weight(Weight::SemiBold)
        .color(theme.secondary_text);

    let side: Align = if on_left { Start.into() } else { End.into() };

    Rectangle::new()
        .width(CLOCK_WIDTH)
        .height(CLOCK_HEIGHT)
        .align_child(side, End)
        .child(Column::new(children![time, date]).align(side).gap(0.0))
}
