use std::process::Command;

use amane::{
    Center, Column, Padding, Pointer, Rectangle, Row,
    Service, SpaceBetween, Text, Weight, Widget, children,
};

use super::{Overlay, PanelView, Region};
use crate::fonts;
use crate::liquid::{self, Blob};
use crate::theme::Theme;

const WIDTH: f32 = 350.0;
const HEIGHT: f32 = 422.0;
const RADIUS: f32 = 28.0;

const EDGE_OVERLAP: f32 = 40.0;
const HIDDEN_MARGIN: f32 = 32.0;

const PADDING: Padding = Padding {
    top: EDGE_OVERLAP + 16.0,
    right: 18.0,
    bottom: 18.0,
    left: 18.0,
};

const INNER_WIDTH: f32 = WIDTH - PADDING.left - PADDING.right;
const GAP: f32 = 12.0;
const CARD_PADDING: f32 = 12.0;
const CARD_INNER_WIDTH: f32 = INNER_WIDTH - CARD_PADDING * 2.0;

#[derive(Clone, Debug, PartialEq)]
pub enum DeviceKind {
    Laptop,
    Keyboard,
    Mouse,
    Headphones,
    Other,
}

#[derive(Clone, Debug)]
pub struct PowerDevice {
    pub name: String,
    pub kind: DeviceKind,
    pub percent: u8,
    #[allow(dead_code)]
    pub state: String,
    pub is_charging: bool,
    pub details: String,
}

#[derive(Clone)]
pub struct PowerDevicesData {
    pub ac_online: bool,
    pub active_profile: String,
    pub laptop_battery: Option<PowerDevice>,
    pub peripherals: Vec<PowerDevice>,
}

impl Default for PowerDevicesData {
    fn default() -> Self {
        Self {
            ac_online: false,
            active_profile: String::from("balanced"),
            laptop_battery: None,
            peripherals: Vec::new(),
        }
    }
}

impl Service for PowerDevicesData {
    fn new() -> Self {
        let mut data = Self::default();
        data.update();
        data
    }

    fn interval() -> std::time::Duration {
        std::time::Duration::from_millis(2500)
    }

    fn update(&mut self) -> bool {
        let (ac_online, laptop, peripherals) = query_power_devices();
        self.ac_online = ac_online;
        self.laptop_battery = laptop;
        self.peripherals = peripherals;
        self.active_profile = query_active_profile();
        true
    }
}

impl PowerDevicesData {
    pub fn set_profile(profile: &'static str) {
        Self::write().active_profile = profile.to_string();

        std::thread::spawn(move || {
            let _ = Command::new("busctl")
                .args([
                    "--system",
                    "set-property",
                    "net.hadess.PowerProfiles",
                    "/net/hadess/PowerProfiles",
                    "net.hadess.PowerProfiles",
                    "ActiveProfile",
                    "s",
                    profile,
                ])
                .status();
        });
    }
}

pub fn view(overlay: &Overlay, theme: &Theme, screen: Region) -> Option<PanelView> {
    let progress = overlay.power_devices.progress.value();

    if progress <= 0.001 {
        return None;
    }

    let width = WIDTH.min(screen.width - 24.0);
    let x = (screen.width - width - 18.0).max(10.0);

    let hidden = -(HEIGHT + HIDDEN_MARGIN);
    let y = -EDGE_OVERLAP + hidden * (1.0 - progress);

    let blob = Blob::new(x, y, width, HEIGHT)
        .radius(RADIUS)
        .child(content(theme, width));

    let input = Region {
        x,
        y,
        width,
        height: HEIGHT,
    };

    let safe_y = y.max(0.0);
    let reach = Region {
        x: x - liquid::CONNECTION,
        y: 0.0,
        width: width + liquid::CONNECTION * 2.0,
        height: (-EDGE_OVERLAP + HEIGHT + liquid::CONNECTION + safe_y).max(1.0),
    };

    Some(PanelView { blob, input, reach })
}

fn content(theme: &Theme, width: f32) -> Rectangle {
    let column = Column::new(children![
        header(theme),
        laptop_card(theme),
        peripherals_card(theme),
    ])
    .gap(GAP)
    .width(INNER_WIDTH);

    Rectangle::new()
        .width(width)
        .height(HEIGHT)
        .padding(PADDING)
        .on_hover(Overlay::hover_power_devices)
        .child(column)
}

fn header(theme: &Theme) -> Row {
    let data = PowerDevicesData::read();

    let icon = Text::new("󰁹")
        .size(19.0)
        .font(fonts::NERD)
        .tight()
        .color(theme.accent);

    let title = Text::new("Power & Devices")
        .size(15.0)
        .font(fonts::BODY)
        .weight(Weight::SemiBold)
        .color(theme.text);

    let left = Row::new(children![icon, title]).gap(8.0).align(Center);

    let (ac_icon, ac_text, ac_color) = if data.ac_online {
        ("󰚥", "AC Connected", theme.success)
    } else {
        ("󰁹", "On Battery", theme.secondary_text)
    };

    let ac_icon_w = Text::new(ac_icon).size(12.0).font(fonts::NERD).tight().color(ac_color);
    let ac_text_w = Text::new(ac_text).size(11.0).font(fonts::BODY).weight(Weight::Medium).color(ac_color);

    let ac_w = ac_text.chars().count() as f32 * 7.0 + 32.0;
    let ac_pill = Rectangle::new()
        .width(ac_w)
        .height(24.0)
        .radius(8.0)
        .fill(theme.surface)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 8.0,
            right: 8.0,
        })
        .align_child(Center, Center)
        .child(Row::new(children![ac_icon_w, ac_text_w]).gap(4.0).align(Center));

    let details_btn = Rectangle::new()
        .width(26.0)
        .height(24.0)
        .radius(8.0)
        .fill(theme.surface)
        .cursor(Pointer)
        .on_click(|_| {
            let _ = Command::new("gnome-power-statistics").spawn();
        })
        .align_child(Center, Center)
        .child(Text::new("󰢮").size(13.0).font(fonts::NERD).tight().color(theme.secondary_text));

    let right = Row::new(children![ac_pill, details_btn]).gap(6.0).align(Center);

    Row::new(children![left, right])
        .width(INNER_WIDTH)
        .justify(SpaceBetween)
        .align(Center)
}

fn laptop_card(theme: &Theme) -> Rectangle {
    let data = PowerDevicesData::read();

    let (name, percent, is_charging, details) = match &data.laptop_battery {
        Some(b) => (b.name.as_str(), b.percent, b.is_charging, b.details.as_str()),
        None => ("Internal Battery", 100, false, "Not detected"),
    };

    let color = if is_charging {
        theme.success
    } else if percent > 20 {
        theme.accent
    } else {
        theme.danger
    };

    let battery_glyph = if is_charging {
        "󰚥"
    } else {
        match percent {
            90.. => "󰁹",
            70..=89 => "󰂁",
            50..=69 => "󰁿",
            30..=49 => "󰁽",
            15..=29 => "󰁻",
            _ => "󰁺",
        }
    };

    let icon_w = Text::new(battery_glyph).size(26.0).font(fonts::NERD).tight().color(color);
    let pct_w = Text::new(format!("{percent}%")).size(20.0).font(fonts::BODY).weight(Weight::Bold).color(theme.text);
    let left_stat = Row::new(children![icon_w, pct_w]).gap(8.0).align(Center);

    let dev_title = Text::new("Laptop Battery").size(13.0).font(fonts::BODY).weight(Weight::SemiBold).color(theme.text);
    let sub_name = Text::new(name).size(10.0).font(fonts::BODY).color(theme.muted_text).elide();
    let detail_color = if is_charging { theme.success } else { theme.accent };
    let status_detail = Text::new(details).size(10.0).font(fonts::BODY).weight(Weight::Medium).color(detail_color);

    let right_info = Column::new(children![dev_title, sub_name, status_detail]).gap(2.0);

    let top_row = Row::new(children![left_stat, right_info])
        .width(CARD_INNER_WIDTH)
        .justify(SpaceBetween)
        .align(Center);

    // Energy Bar
    let bar_height: f32 = 6.0;
    let track = Rectangle::new()
        .width(CARD_INNER_WIDTH)
        .height(bar_height)
        .radius(bar_height / 2.0)
        .fill(theme.border);

    let fill_w = ((f32::from(percent) / 100.0) * CARD_INNER_WIDTH).clamp(6.0, CARD_INNER_WIDTH);
    let filled = Rectangle::new()
        .width(fill_w)
        .height(bar_height)
        .radius(bar_height / 2.0)
        .fill(color);

    let bar = amane::Stack::new(children![track, filled]).width(CARD_INNER_WIDTH).height(bar_height);
    let switcher = power_profile_switcher(theme, &data.active_profile);

    let card_content = Column::new(children![top_row, bar, switcher]).gap(9.0);

    Rectangle::new()
        .width(INNER_WIDTH)
        .height(138.0)
        .radius(16.0)
        .fill(theme.surface)
        .padding(Padding {
            top: CARD_PADDING,
            right: CARD_PADDING,
            bottom: CARD_PADDING,
            left: CARD_PADDING,
        })
        .child(card_content)
}

fn power_profile_switcher(theme: &Theme, active_profile: &str) -> Row {
    let btn_w = (CARD_INNER_WIDTH - 8.0) / 3.0;

    let p_saver = profile_button(
        theme,
        "power-saver",
        "Power Saver",
        "󰌪",
        active_profile == "power-saver",
        btn_w,
    );

    let p_balanced = profile_button(
        theme,
        "balanced",
        "Balanced",
        "󰾅",
        active_profile == "balanced",
        btn_w,
    );

    let p_perf = profile_button(
        theme,
        "performance",
        "Performance",
        "󰓅",
        active_profile == "performance",
        btn_w,
    );

    Row::new(children![p_saver, p_balanced, p_perf])
        .width(CARD_INNER_WIDTH)
        .gap(4.0)
        .align(Center)
}

fn profile_button(
    theme: &Theme,
    profile_id: &'static str,
    label: &str,
    icon: &str,
    is_active: bool,
    width: f32,
) -> Rectangle {
    let (fill, icon_color, text_color, weight) = if is_active {
        (theme.selected_surface, theme.accent, theme.text, Weight::SemiBold)
    } else {
        (amane::Color::TRANSPARENT, theme.secondary_text, theme.secondary_text, Weight::Medium)
    };

    let icon_w = Text::new(icon)
        .size(11.0)
        .font(fonts::NERD)
        .tight()
        .color(icon_color);

    let label_w = Text::new(label)
        .size(10.0)
        .font(fonts::BODY)
        .weight(weight)
        .color(text_color)
        .elide();

    let row = Row::new(children![icon_w, label_w])
        .gap(4.0)
        .align(Center);

    Rectangle::new()
        .width(width)
        .height(24.0)
        .radius(7.0)
        .fill(fill)
        .cursor(Pointer)
        .on_click(move |_| {
            PowerDevicesData::set_profile(profile_id);
        })
        .align_child(Center, Center)
        .child(row)
}

fn peripherals_card(theme: &Theme) -> Rectangle {
    let data = PowerDevicesData::read();

    let header_icon = Text::new("󰂯").size(13.0).font(fonts::NERD).tight().color(theme.secondary_text);
    let count_text = format!("Connected Devices ({})", data.peripherals.len());
    let header_title = Text::new(&count_text).size(12.0).font(fonts::BODY).weight(Weight::SemiBold).color(theme.secondary_text);
    let header_row = Row::new(children![header_icon, header_title]).gap(6.0).align(Center);

    let mut items: Vec<Box<dyn Widget>> = Vec::new();
    items.push(Box::new(header_row));

    if data.peripherals.is_empty() {
        let empty_text = Text::new("No wireless peripherals connected")
            .size(11.0)
            .font(fonts::BODY)
            .color(theme.muted_text);

        let empty_row = Rectangle::new()
            .width(CARD_INNER_WIDTH)
            .height(50.0)
            .align_child(Center, Center)
            .child(empty_text);

        items.push(Box::new(empty_row));
    } else {
        for dev in data.peripherals.iter().take(3) {
            items.push(Box::new(peripheral_row(theme, dev)));
        }
    }

    let card_content = Column::new(items).gap(8.0);

    Rectangle::new()
        .width(INNER_WIDTH)
        .height(180.0)
        .radius(16.0)
        .fill(theme.surface)
        .padding(Padding {
            top: CARD_PADDING,
            right: CARD_PADDING,
            bottom: CARD_PADDING,
            left: CARD_PADDING,
        })
        .child(card_content)
}

fn peripheral_row(theme: &Theme, dev: &PowerDevice) -> Column {
    let icon_str = match dev.kind {
        DeviceKind::Keyboard => "󰌌",
        DeviceKind::Mouse => "󰍽",
        DeviceKind::Headphones => "󰋋",
        _ => "󰂯",
    };

    let dev_color = if dev.percent > 20 {
        theme.accent
    } else {
        theme.danger
    };

    let icon_w = Text::new(icon_str).size(16.0).font(fonts::NERD).tight().color(theme.accent);
    let name_w = Text::new(&dev.name).size(12.0).font(fonts::BODY).weight(Weight::SemiBold).color(theme.text).elide();
    let detail_w = Text::new(&dev.details).size(9.0).font(fonts::BODY).color(theme.muted_text);

    let text_col = Column::new(children![name_w, detail_w]).gap(1.0);
    let left = Row::new(children![icon_w, text_col]).gap(8.0).align(Center);

    // Battery percentage pill on the right
    let batt_icon = Text::new("󰁹").size(10.0).font(fonts::NERD).tight().color(dev_color);
    let pct_label = Text::new(format!("{}%", dev.percent)).size(11.0).font(fonts::BODY).weight(Weight::SemiBold).color(dev_color);
    let pill_row = Row::new(children![batt_icon, pct_label]).gap(4.0).align(Center);

    let right_pill = Rectangle::new()
        .width(48.0)
        .height(22.0)
        .radius(6.0)
        .fill(theme.selected_surface)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 6.0,
            right: 6.0,
        })
        .align_child(Center, Center)
        .child(pill_row);

    let top_line = Row::new(children![left, right_pill])
        .width(CARD_INNER_WIDTH)
        .justify(SpaceBetween)
        .align(Center);

    // Mini battery track
    let bar_height: f32 = 3.0;
    let track = Rectangle::new()
        .width(CARD_INNER_WIDTH)
        .height(bar_height)
        .radius(bar_height / 2.0)
        .fill(theme.border);

    let fill_w = ((f32::from(dev.percent) / 100.0) * CARD_INNER_WIDTH).clamp(3.0, CARD_INNER_WIDTH);
    let filled = Rectangle::new()
        .width(fill_w)
        .height(bar_height)
        .radius(bar_height / 2.0)
        .fill(dev_color);

    let mini_bar = amane::Stack::new(children![track, filled]).width(CARD_INNER_WIDTH).height(bar_height);

    Column::new(children![top_line, mini_bar]).gap(4.0)
}

fn query_power_devices() -> (bool, Option<PowerDevice>, Vec<PowerDevice>) {
    let Ok(output) = Command::new("upower").arg("-d").output() else {
        return (false, None, Vec::new());
    };

    let text = String::from_utf8_lossy(&output.stdout);

    let mut ac_online = false;
    let mut laptop_battery = None;
    let mut peripherals = Vec::new();

    let mut current_path = String::new();
    let mut model = String::new();
    let mut dev_type = String::new();
    let mut state = String::new();
    let mut percent: Option<u8> = None;
    let mut rate = String::new();
    let mut time_str = String::new();

    let flush_device = |path: &str,
                        model: &str,
                        dev_type: &str,
                        state: &str,
                        percent: Option<u8>,
                        rate: &str,
                        time_str: &str,
                        laptop_battery: &mut Option<PowerDevice>,
                        peripherals: &mut Vec<PowerDevice>| {
        let Some(p) = percent else { return };
        if path.contains("DisplayDevice") {
            return;
        }

        let is_charging = state.eq_ignore_ascii_case("charging");
        let name = if !model.is_empty() {
            model.to_string()
        } else if path.contains("battery_BAT") {
            String::from("Internal Battery")
        } else {
            String::from("Wireless Device")
        };

        if path.contains("battery_BAT") || dev_type == "battery" && laptop_battery.is_none() {
            let mut details = String::new();
            if !rate.is_empty() {
                details.push_str(rate);
            }
            if !time_str.is_empty() {
                if !details.is_empty() {
                    details.push_str(" • ");
                }
                details.push_str(time_str);
            }
            if details.is_empty() {
                details = state.to_string();
            }

            *laptop_battery = Some(PowerDevice {
                name,
                kind: DeviceKind::Laptop,
                percent: p,
                state: state.to_string(),
                is_charging,
                details,
            });
        } else {
            let kind = if dev_type == "keyboard"
                || path.contains("keyboard")
                || name.to_lowercase().contains("keyboard")
                || name.contains("BT-1")
            {
                DeviceKind::Keyboard
            } else if dev_type == "mouse"
                || path.contains("mouse")
                || name.to_lowercase().contains("mouse")
                || name.contains("M575")
            {
                DeviceKind::Mouse
            } else if dev_type == "headphones"
                || dev_type == "headset"
                || name.to_lowercase().contains("headphone")
                || name.to_lowercase().contains("buds")
                || name.to_lowercase().contains("airpods")
            {
                DeviceKind::Headphones
            } else {
                DeviceKind::Other
            };

            let details = match kind {
                DeviceKind::Keyboard => String::from("Wireless Keyboard"),
                DeviceKind::Mouse => String::from("Wireless Mouse"),
                DeviceKind::Headphones => String::from("Bluetooth Audio"),
                _ => String::from("Connected Peripheral"),
            };

            peripherals.push(PowerDevice {
                name,
                kind,
                percent: p,
                state: state.to_string(),
                is_charging,
                details,
            });
        }
    };

    for line in text.lines() {
        let trimmed = line.trim();
        if line.starts_with("Device: ") {
            flush_device(
                &current_path,
                &model,
                &dev_type,
                &state,
                percent,
                &rate,
                &time_str,
                &mut laptop_battery,
                &mut peripherals,
            );
            current_path = line.trim_start_matches("Device: ").trim().to_string();
            model.clear();
            dev_type.clear();
            state.clear();
            percent = None;
            rate.clear();
            time_str.clear();
        } else if trimmed.starts_with("model:") {
            if let Some((_, v)) = trimmed.split_once(':') {
                model = v.trim().to_string();
            }
        } else if trimmed.starts_with("state:") {
            if let Some((_, v)) = trimmed.split_once(':') {
                state = v.trim().to_string();
            }
        } else if trimmed.starts_with("percentage:") {
            if let Some((_, v)) = trimmed.split_once(':') {
                let s = v.trim().trim_end_matches('%');
                if let Ok(num) = s.parse::<f32>() {
                    percent = Some(num.round() as u8);
                }
            }
        } else if trimmed.starts_with("energy-rate:") {
            if let Some((_, v)) = trimmed.split_once(':') {
                rate = v.trim().to_string();
            }
        } else if trimmed.starts_with("time to full:") || trimmed.starts_with("time to empty:") {
            if let Some((_, v)) = trimmed.split_once(':') {
                time_str = v.trim().to_string();
            }
        } else if trimmed.starts_with("online:") {
            if let Some((_, v)) = trimmed.split_once(':') {
                if v.trim().eq_ignore_ascii_case("yes") {
                    ac_online = true;
                }
            }
        } else if matches!(trimmed, "battery" | "keyboard" | "mouse" | "headphones" | "headset") {
            dev_type = trimmed.to_string();
        }
    }

    flush_device(
        &current_path,
        &model,
        &dev_type,
        &state,
        percent,
        &rate,
        &time_str,
        &mut laptop_battery,
        &mut peripherals,
    );

    (ac_online, laptop_battery, peripherals)
}

pub fn ipc(_arguments: &[String]) -> String {
    Overlay::toggle_power_devices();
    String::from("ok")
}

fn query_active_profile() -> String {
    let Ok(output) = Command::new("busctl")
        .args([
            "--system",
            "get-property",
            "net.hadess.PowerProfiles",
            "/net/hadess/PowerProfiles",
            "net.hadess.PowerProfiles",
            "ActiveProfile",
        ])
        .output()
    else {
        return String::from("balanced");
    };

    let text = String::from_utf8_lossy(&output.stdout);
    text.split_whitespace()
        .last()
        .map(|s| s.trim_matches('"').to_string())
        .unwrap_or_else(|| String::from("balanced"))
}
