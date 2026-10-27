use std::process::Command;
use std::time::Duration;

use amane::{
    Center, Column, Padding, Pointer, Rectangle, Row, ScrollArea, Service,
    SpaceBetween, Start, Text, Weight, Widget, children,
};

use super::{Overlay, PanelView, Region};
use crate::fonts;
use crate::liquid::Blob;
use crate::theme::{self, Theme};

const WIDTH: f32 = 330.0;
const HEIGHT: f32 = 310.0;
const RADIUS: f32 = 22.0;
const TOP_OFFSET: f32 = 8.0;
const SHADOW_MARGIN: f32 = 24.0;
const HIDDEN_MARGIN: f32 = 32.0;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct HaConfig {
    pub url: String,
    pub token: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HaLight {
    pub entity_id: String,
    pub friendly_name: String,
    pub state: bool,
    pub brightness: Option<u8>,
}

#[derive(Clone)]
pub struct HomeAssistantData {
    pub reachable: bool,
    pub lights: Vec<HaLight>,
    pub url: String,
    pub token: String,
}

impl Default for HomeAssistantData {
    fn default() -> Self {
        Self {
            reachable: false,
            lights: Vec::new(),
            url: String::new(),
            token: String::new(),
        }
    }
}

fn load_config() -> Option<HaConfig> {
    let home = std::env::var("HOME").unwrap_or_else(|_| String::from("/home/rcrout"));
    let path = format!("{home}/.config/amane/homeassistant.json");
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

impl Service for HomeAssistantData {
    fn new() -> Self {
        let (url, token) = match load_config() {
            Some(cfg) => (cfg.url, cfg.token),
            None => (String::new(), String::new()),
        };

        let mut data = Self {
            reachable: false,
            lights: Vec::new(),
            url,
            token,
        };

        if !data.url.is_empty() && !data.token.is_empty() {
            data.refresh();
        }

        data
    }

    fn interval() -> Duration {
        Duration::from_secs(3)
    }

    fn update(&mut self) -> bool {
        if self.url.is_empty() || self.token.is_empty() {
            if let Some(cfg) = load_config() {
                self.url = cfg.url;
                self.token = cfg.token;
            } else {
                return false;
            }
        }

        self.refresh()
    }
}

impl HomeAssistantData {
    pub fn refresh(&mut self) -> bool {
        let endpoint = format!("{}/api/states", self.url);
        let auth_header = format!("Authorization: Bearer {}", self.token);

        let output = Command::new("curl")
            .args([
                "-sf",
                "--connect-timeout", "1",
                "--max-time", "2",
                "-H", &auth_header,
                "-H", "Content-Type: application/json",
                &endpoint,
            ])
            .output();

        let Ok(out) = output else {
            self.reachable = false;
            return true;
        };

        if !out.status.success() {
            self.reachable = false;
            return true;
        }

        let Ok(json_val) = serde_json::from_slice::<serde_json::Value>(&out.stdout) else {
            self.reachable = false;
            return true;
        };

        let Some(arr) = json_val.as_array() else {
            self.reachable = false;
            return true;
        };

        let mut lights = Vec::new();
        for item in arr {
            let entity_id = item.get("entity_id").and_then(|v| v.as_str()).unwrap_or("");
            if entity_id.starts_with("light.") {
                let state_str = item.get("state").and_then(|v| v.as_str()).unwrap_or("off");
                let state = state_str == "on";
                let attrs = item.get("attributes");
                let friendly_name = attrs
                    .and_then(|a| a.get("friendly_name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or(entity_id)
                    .to_string();
                let brightness = attrs
                    .and_then(|a| a.get("brightness"))
                    .and_then(|v| v.as_u64())
                    .map(|b| b as u8);

                lights.push(HaLight {
                    entity_id: entity_id.to_string(),
                    friendly_name,
                    state,
                    brightness,
                });
            }
        }

        lights.sort_by(|a, b| a.friendly_name.cmp(&b.friendly_name));

        self.reachable = true;
        self.lights = lights;
        true
    }

    pub fn toggle_light(entity_id: &str) {
        let (url, token) = {
            let mut data = Self::write();
            for light in &mut data.lights {
                if light.entity_id == entity_id {
                    light.state = !light.state;
                    break;
                }
            }
            (data.url.clone(), data.token.clone())
        };

        let target = entity_id.to_string();
        std::thread::spawn(move || {
            let endpoint = format!("{url}/api/services/light/toggle");
            let auth = format!("Authorization: Bearer {token}");
            let body = format!(r#"{{"entity_id":"{target}"}}"#);

            let _ = Command::new("curl")
                .args([
                    "-sf",
                    "--connect-timeout", "1",
                    "--max-time", "3",
                    "-X", "POST",
                    "-H", &auth,
                    "-H", "Content-Type: application/json",
                    "-d", &body,
                    &endpoint,
                ])
                .output();
        });
    }

    pub fn turn_off_all_lights() {
        let (url, token) = {
            let mut data = Self::write();
            for light in &mut data.lights {
                light.state = false;
            }
            (data.url.clone(), data.token.clone())
        };

        std::thread::spawn(move || {
            let endpoint = format!("{url}/api/services/light/turn_off");
            let auth = format!("Authorization: Bearer {token}");
            let body = r#"{"entity_id":"all"}"#;

            let _ = Command::new("curl")
                .args([
                    "-sf",
                    "--connect-timeout", "1",
                    "--max-time", "3",
                    "-X", "POST",
                    "-H", &auth,
                    "-H", "Content-Type: application/json",
                    "-d", body,
                    &endpoint,
                ])
                .output();
        });
    }
}

pub fn view(overlay: &Overlay, theme: &Theme, screen: Region) -> Option<PanelView> {
    let progress = overlay.home_assistant.progress.value();

    if !overlay.home_assistant.shown && progress <= 0.001 {
        return None;
    }

    let width = WIDTH.min(screen.width - 24.0);
    // Align flyout slightly to the right of center, directly under the right-wing pills
    let center_x = (screen.width - width) / 2.0;
    let x = (center_x + 90.0).min(screen.width - width - 16.0);

    let hidden = -(HEIGHT + HIDDEN_MARGIN);
    let y = TOP_OFFSET + (hidden - TOP_OFFSET) * (1.0 - progress);

    let blob = Blob::new(x, y, width, HEIGHT)
        .radius(RADIUS)
        .child(content(overlay, theme, width));

    let input = Region {
        x,
        y,
        width,
        height: HEIGHT,
    };

    let reach = Region {
        x: x - SHADOW_MARGIN,
        y: 0.0,
        width: width + SHADOW_MARGIN * 2.0,
        height: TOP_OFFSET + HEIGHT + SHADOW_MARGIN,
    };

    Some(PanelView { blob, input, reach })
}

fn content(_overlay: &Overlay, theme: &Theme, width: f32) -> Rectangle {
    let data = HomeAssistantData::read();
    let on_count = data.lights.iter().filter(|l| l.state).count();

    let header = header_view(theme, width - 24.0, on_count);
    let divider = Rectangle::new()
        .width(width - 24.0)
        .height(1.0)
        .fill(theme::with_opacity(theme.border, 0.45));

    let list_w = width - 20.0;
    let list_h = HEIGHT - 32.0 - 1.0 - 18.0 - 20.0;
    let lights_list = lights_scroll(theme, &data.lights, list_w, list_h);

    let col = Column::new(children![header, divider, lights_list])
        .gap(8.0)
        .align(Center);

    Rectangle::new()
        .width(width)
        .height(HEIGHT)
        .padding(Padding {
            top: 12.0,
            bottom: 10.0,
            left: 10.0,
            right: 10.0,
        })
        .on_hover(Overlay::hover_home_assistant)
        .child(col)
}

fn header_view(theme: &Theme, width: f32, on_count: usize) -> Row {
    let icon = Text::new("󰌵")
        .size(15.0)
        .font(fonts::NERD)
        .tight()
        .color(if on_count > 0 { theme.accent } else { theme.secondary_text });

    let title = Text::new("Lights")
        .size(13.0)
        .font(fonts::BODY)
        .weight(Weight::SemiBold)
        .color(theme.text);

    let badge_text = if on_count > 0 {
        format!("{on_count} on")
    } else {
        String::from("All off")
    };

    let badge = Text::new(&badge_text)
        .size(11.0)
        .font(fonts::BODY)
        .color(theme.secondary_text);

    let left = Row::new(children![icon, title, badge]).gap(6.0).align(Center);

    let off_btn = Rectangle::new()
        .width(64.0)
        .height(24.0)
        .radius(8.0)
        .fill(if on_count > 0 { theme.selected_surface } else { amane::Color::TRANSPARENT })
        .cursor(Pointer)
        .on_click(|_| HomeAssistantData::turn_off_all_lights())
        .align_child(Center, Center)
        .child(
            Text::new("All Off")
                .size(11.0)
                .font(fonts::BODY)
                .weight(Weight::Medium)
                .color(if on_count > 0 { theme.text } else { theme.muted_text }),
        );

    Row::new(children![left, off_btn])
        .width(width)
        .justify(SpaceBetween)
        .align(Center)
}

fn lights_scroll(theme: &Theme, lights: &[HaLight], width: f32, height: f32) -> ScrollArea {
    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    for light in lights {
        rows.push(Box::new(light_card(theme, light, width)));
    }

    let col = Column::new(rows).width(width).gap(4.0);

    ScrollArea::new("ha-lights-scroll", col)
        .width(width)
        .height(height)
}

fn light_card(theme: &Theme, light: &HaLight, width: f32) -> Rectangle {
    let icon_color = if light.state {
        theme.accent
    } else {
        theme.muted_text
    };

    let icon = Text::new("󰌵")
        .size(14.0)
        .font(fonts::NERD)
        .tight()
        .color(icon_color);

    let name = Text::new(&light.friendly_name)
        .size(12.0)
        .font(fonts::BODY)
        .weight(if light.state { Weight::SemiBold } else { Weight::Regular })
        .color(theme.text)
        .elide();

    let name_box = Rectangle::new()
        .width((width - 24.0 - 46.0 - 16.0).max(80.0))
        .height(20.0)
        .align_child(Start, Center)
        .child(name);

    let switch_w = 40.0;
    let switch_h = 22.0;
    let knob_size = if light.state { 16.0 } else { 14.0 };
    let knob_x = if light.state { switch_w - knob_size - 3.0 } else { 3.0 };

    let knob = Rectangle::new()
        .width(knob_size)
        .height(knob_size)
        .radius(knob_size / 2.0)
        .fill(if light.state { theme.on_accent } else { theme.secondary_text })
        .translate(knob_x, (switch_h - knob_size) / 2.0);

    let switch = Rectangle::new()
        .width(switch_w)
        .height(switch_h)
        .radius(switch_h / 2.0)
        .fill(if light.state { theme.accent } else { theme.selected_surface })
        .border(if light.state { 0.0 } else { 1.5 }, theme.border)
        .child(knob);

    let left = Row::new(children![icon, name_box]).gap(8.0).align(Center);

    let content = Row::new(children![left, switch])
        .width(width - 16.0)
        .justify(SpaceBetween)
        .align(Center);

    let entity_id = light.entity_id.clone();

    Rectangle::new()
        .width(width)
        .height(34.0)
        .radius(10.0)
        .fill(if light.state { theme::with_opacity(theme.surface, 0.9) } else { amane::Color::TRANSPARENT })
        .padding(Padding {
            top: 4.0,
            bottom: 4.0,
            left: 8.0,
            right: 8.0,
        })
        .cursor(Pointer)
        .on_click(move |_| {
            HomeAssistantData::toggle_light(&entity_id);
        })
        .align_child(Center, Center)
        .child(content)
}

pub fn ipc(arguments: &[String]) -> String {
    let command = arguments.first().map(String::as_str).unwrap_or("toggle");
    let monitor_arg = arguments.get(1).map(String::as_str);

    let shown = Overlay::read().home_assistant.shown;

    let wanted = match command {
        "show" => true,
        "hide" => false,
        _ => !shown,
    };

    if wanted {
        Overlay::write().home_assistant.show();
        if let Some(mon) = monitor_arg {
            Overlay::write().set_active_monitor(Some(mon));
        }
    } else {
        Overlay::write().home_assistant.hide();
    }

    String::from("ok")
}
