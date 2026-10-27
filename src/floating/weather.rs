mod csv;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use amane::Service;

use crate::settings::Settings;

const FORECAST: &str = "https://api.open-meteo.com/v1/forecast";
const AIR_QUALITY: &str = "https://air-quality-api.open-meteo.com/v1/air-quality";

// how often the loop looks at the settings and the refresh button between polls
const CHECK: Duration = Duration::from_secs(1);

// set by the refresh button, the next check asks at once
static REFRESH: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Default, PartialEq)]
pub struct ForecastDay {
    pub day_name: String,
    pub code: u32,
    pub high: f32,
    pub low: f32,
    pub is_today: bool,
}

// the latest reading for the saved location, empty until the first answer
#[derive(Default, PartialEq)]
pub struct Weather {
    pub place: String,

    pub temperature: Option<f32>,
    pub high: Option<f32>,
    pub low: Option<f32>,

    // a wmo weather code, like 3 for cloudy
    pub code: Option<u32>,

    pub humidity: Option<f32>,
    pub uv_index: Option<f32>,

    // the us air quality index, 0 to 500
    pub air_quality: Option<f32>,

    pub forecast_7d: Vec<ForecastDay>,
}

/*
 * polled, and asked outside the lock: the answer comes over the network,
 * and nothing is written when it didn't change
 */
impl Service for Weather {
    fn new() -> Self {
        Self::default()
    }

    // asks again once the interval is up, the location or unit changes, or refresh is pressed
    fn listen() {
        loop {
            let asked = request();

            let fresh = fetch(&asked);

            if *Self::read() != fresh {
                *Self::write() = fresh;
            }

            let started = Instant::now();

            loop {
                thread::sleep(CHECK);

                let minutes = Settings::read().number("weather_minutes").max(1.0);

                let due = started.elapsed().as_secs_f32() >= minutes * 60.0;

                if due || request() != asked || REFRESH.swap(false, Ordering::Relaxed) {
                    break;
                }
            }
        }
    }
}

impl Weather {
    // from the settings' refresh button
    pub fn refresh() {
        REFRESH.store(true, Ordering::Relaxed);
    }

    pub fn condition(&self) -> &'static str {
        weather_code_condition(self.code.unwrap_or(0))
    }

    // material design weather glyphs from the nerd font
    pub fn icon(&self) -> &'static str {
        weather_code_icon(self.code.unwrap_or(0))
    }
}

pub fn weather_code_condition(code: u32) -> &'static str {
    match code {
        0 => "Clear sky",
        1 => "Mainly clear",
        2 => "Partly cloudy",
        3 => "Cloudy",
        45 | 48 => "Fog",
        51 | 53 | 55 => "Drizzle",
        56 | 57 => "Freezing drizzle",
        61 => "Light rain",
        63 => "Rain",
        65 => "Heavy rain",
        66 | 67 => "Freezing rain",
        71 | 73 | 75 | 77 => "Snow",
        80 | 81 | 82 => "Rain showers",
        85 | 86 => "Snow showers",
        95 | 96 | 99 => "Thunderstorm",
        _ => "Clear sky",
    }
}

pub fn weather_code_icon(code: u32) -> &'static str {
    match code {
        0 => "\u{f0599}",
        1 | 2 => "\u{f0595}",
        3 => "\u{f0590}",
        45 | 48 => "\u{f0591}",
        51..=57 | 80..=82 => "\u{f0596}",
        61..=67 => "\u{f0597}",
        71..=77 => "\u{f0598}",
        85 | 86 => "\u{f0f36}",
        95.. => "\u{f067e}",
        _ => "\u{f0599}",
    }
}

// where to ask about and in which unit, as the settings say
#[derive(PartialEq)]
struct Request {
    location: Option<Location>,
    fahrenheit: bool,
}

#[derive(PartialEq)]
struct Location {
    latitude: String,
    longitude: String,
    name: String,
}

fn fetch(request: &Request) -> Weather {
    let Some(location) = &request.location else {
        return Weather {
            place: String::from("Set a location"),
            ..Weather::default()
        };
    };

    let coordinates = format!("latitude={}&longitude={}", location.latitude, location.longitude);

    let unit = if request.fahrenheit { "fahrenheit" } else { "celsius" };

    let forecast_url = format!(
        "{FORECAST}?{coordinates}&current=temperature_2m,relative_humidity_2m,weather_code,uv_index\
         &daily=weather_code,temperature_2m_max,temperature_2m_min&temperature_unit={unit}\
         &forecast_days=7&timezone=auto"
    );

    let forecast_json_text = amane::output(&format!("curl -sf --max-time 20 '{forecast_url}'"));
    let json_val: Option<serde_json::Value> = serde_json::from_str(&forecast_json_text).ok();

    let mut forecast_7d = Vec::new();
    let mut temp = None;
    let mut high = None;
    let mut low = None;
    let mut code = None;
    let mut humidity = None;
    let mut uv_index = None;

    if let Some(val) = &json_val {
        temp = val["current"]["temperature_2m"].as_f64().map(|f| f as f32);
        code = val["current"]["weather_code"].as_u64().map(|u| u as u32);
        humidity = val["current"]["relative_humidity_2m"].as_f64().map(|f| f as f32);
        uv_index = val["current"]["uv_index"].as_f64().map(|f| f as f32);

        if let (Some(times), Some(codes), Some(maxs), Some(mins)) = (
            val["daily"]["time"].as_array(),
            val["daily"]["weather_code"].as_array(),
            val["daily"]["temperature_2m_max"].as_array(),
            val["daily"]["temperature_2m_min"].as_array(),
        ) {
            for i in 0..times.len().min(7) {
                let date_str = times[i].as_str().unwrap_or("");
                let dcode = codes.get(i).and_then(|c| c.as_u64()).unwrap_or(0) as u32;
                let dhigh = maxs.get(i).and_then(|m| m.as_f64()).unwrap_or(70.0) as f32;
                let dlow = mins.get(i).and_then(|m| m.as_f64()).unwrap_or(50.0) as f32;

                let day_name = if let Some((y, m, d)) = parse_ymd(date_str) {
                    let days = crate::clock::days_since_1970(y, m, d);
                    match days.rem_euclid(7) {
                        0 => "Thu",
                        1 => "Fri",
                        2 => "Sat",
                        3 => "Sun",
                        4 => "Mon",
                        5 => "Tue",
                        _ => "Wed",
                    }.to_string()
                } else {
                    format!("Day {}", i + 1)
                };

                forecast_7d.push(ForecastDay {
                    day_name,
                    code: dcode,
                    high: dhigh,
                    low: dlow,
                    is_today: i == 0,
                });
            }
        }
    }

    if let Some(first) = forecast_7d.first() {
        if high.is_none() {
            high = Some(first.high);
        }
        if low.is_none() {
            low = Some(first.low);
        }
    }

    let air = get(&format!("{AIR_QUALITY}?{coordinates}&current=us_aqi&format=csv"));
    let number = |values: &HashMap<String, String>, name: &str| values.get(name)?.parse().ok();

    Weather {
        place: location.name.clone(),
        temperature: temp,
        high,
        low,
        code,
        humidity,
        uv_index,
        air_quality: number(&air, "us_aqi"),
        forecast_7d,
    }
}

fn parse_ymd(s: &str) -> Option<(i64, i64, i64)> {
    let mut parts = s.split('-');
    let y = parts.next()?.parse().ok()?;
    let m = parts.next()?.parse().ok()?;
    let d = parts.next()?.parse().ok()?;
    Some((y, m, d))
}

// empty when offline; the next poll tries again
fn get(url: &str) -> HashMap<String, String> {
    let text = amane::output(&format!("curl -sf --max-time 20 '{url}'"));

    csv::values(&text)
}

fn request() -> Request {
    let settings = Settings::read();

    // the coordinates go into a shell command, so only numbers are let through
    let number = |key: &str| {
        let value = settings.text(key).trim();

        value.parse::<f64>().ok()?;

        Some(String::from(value))
    };

    let location = match (number("weather_latitude"), number("weather_longitude")) {
        (Some(latitude), Some(longitude)) => {
            let name = settings.text("weather_place").trim();

            let name = if name.is_empty() { "Weather" } else { name };

            Some(Location {
                latitude,
                longitude,
                name: String::from(name),
            })
        }

        _ => None,
    };

    Request {
        location,
        fahrenheit: settings.text("weather_unit") == "fahrenheit",
    }
}
