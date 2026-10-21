use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{LazyLock, Mutex};
use std::time::Instant;

use amane::{
    Brightness, Center, Color, End, Column, Full, Padding, Point, Pointer, Rectangle, Row, ScrollArea, Service,
    SpaceBetween, Stack, Start, Text, Weight, Widget, children,
};

use super::super::button::{self, Style};
use super::super::page::Page;
use super::super::{high_surface, hover, hovered, row};
use crate::fonts;
use crate::theme::Theme;

const CANVAS_HEIGHT: f32 = 228.0;
const CANVAS_PAD: f32 = 28.0;
const SNAP_PX: f32 = 16.0;

const COMMON_SCALES: [f32; 7] = [1.0, 1.25, 1.5, 1.75, 2.0, 2.5, 3.0];
const TRANSFORMS: [(&str, u8); 4] = [("Normal", 0), ("90°", 1), ("180°", 2), ("270°", 3)];

fn fmt_scale(s: f32) -> String {
    let formatted = format!("{s:.2}");
    let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
    format!("{trimmed}×")
}

#[derive(Clone, Debug, PartialEq)]
pub struct Mode {
    pub width: i32,
    pub height: i32,
    pub refresh_mhz: i32,
}

impl Mode {
    pub fn refresh_hz(&self) -> f32 {
        self.refresh_mhz as f32 / 1000.0
    }

    pub fn label(&self) -> String {
        let hz = self.refresh_hz();
        if (hz - hz.round()).abs() < 0.05 {
            format!("{} × {} @ {:.0} Hz", self.width, self.height, hz)
        } else {
            format!("{} × {} @ {:.2} Hz", self.width, self.height, hz)
        }
    }
}

#[derive(Clone, Debug)]
pub struct OutputInfo {
    pub name: String,
    pub ident: String,
    pub description: String,
    pub width: i32,
    pub height: i32,
    pub refresh_mhz: i32,
    pub modes: Vec<Mode>,
    pub x: i32,
    pub y: i32,
    pub scale: f32,
    pub transform: u8,
    pub disabled: bool,
}

impl OutputInfo {
    pub fn logical_size(&self) -> (i32, i32) {
        let (w, h) = if matches!(self.transform, 1 | 3) {
            (self.height, self.width)
        } else {
            (self.width, self.height)
        };
        let s = self.scale.max(0.25);
        (
            ((w as f32) / s).round() as i32,
            ((h as f32) / s).round() as i32,
        )
    }

    pub fn current_mode_index(&self) -> usize {
        self.modes
            .iter()
            .position(|m| {
                m.width == self.width
                    && m.height == self.height
                    && (m.refresh_mhz - self.refresh_mhz).abs() < 100
            })
            .unwrap_or(0)
    }
}

struct DragSession {
    index: usize,
    offset_x: f32,
    offset_y: f32,
    last_tick: Instant,
}

pub struct State {
    pub outputs: Vec<OutputInfo>,
    pub selected: usize,
    pub status: String,
    pub mode_menu_open: bool,
    drag: Option<DragSession>,
}

static STATE: LazyLock<Mutex<State>> = LazyLock::new(|| {
    let outputs = query_niri_outputs();
    Mutex::new(State {
        outputs,
        selected: 0,
        status: String::from("Drag displays on the canvas to arrange them, then click Save Layout."),
        mode_menu_open: false,
        drag: None,
    })
});

pub fn reload() {
    if let Ok(mut state) = STATE.lock() {
        state.outputs = query_niri_outputs();
        if state.selected >= state.outputs.len() {
            state.selected = 0;
        }
        state.mode_menu_open = false;
        state.drag = None;
        state.status = String::from("Reloaded current display configuration from Niri.");
    }
}

fn query_niri_outputs() -> Vec<OutputInfo> {
    let Ok(out) = Command::new("niri").args(["msg", "--json", "outputs"]).output() else {
        return Vec::new();
    };
    if !out.status.success() {
        return Vec::new();
    }
    let Ok(json) = serde_json::from_slice::<serde_json::Value>(&out.stdout) else {
        return Vec::new();
    };
    let Some(map) = json.as_object() else {
        return Vec::new();
    };

    let mut list: Vec<OutputInfo> = Vec::new();

    for (name, item) in map {
        let raw_modes: Vec<Mode> = item
            .get("modes")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| {
                        Some(Mode {
                            width: m.get("width")?.as_i64()? as i32,
                            height: m.get("height")?.as_i64()? as i32,
                            refresh_mhz: m.get("refresh_rate")?.as_i64()? as i32,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let cur_idx = item
            .get("current_mode")
            .and_then(|v| v.as_u64())
            .map(|i| i as usize);

        let active_mode = cur_idx
            .and_then(|i| raw_modes.get(i).cloned())
            .or_else(|| raw_modes.first().cloned())
            .unwrap_or(Mode {
                width: 1920,
                height: 1080,
                refresh_mhz: 60000,
            });

        let mut modes = raw_modes;
        modes.sort_by(|a, b| {
            (b.width * b.height)
                .cmp(&(a.width * a.height))
                .then_with(|| b.refresh_mhz.cmp(&a.refresh_mhz))
        });
        modes.dedup_by(|a, b| {
            a.width == b.width
                && a.height == b.height
                && (a.refresh_mhz - b.refresh_mhz).abs() < 50
        });

        let logical = item.get("logical").filter(|l| !l.is_null());
        let (x, y, scale, transform, disabled) = if let Some(loc) = logical {
            let x = loc.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let y = loc.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let scale = loc.get("scale").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
            let trans_str = loc
                .get("transform")
                .and_then(|v| v.as_str())
                .unwrap_or("Normal")
                .to_ascii_lowercase();
            let transform = match trans_str.as_str() {
                "90" => 1,
                "180" => 2,
                "270" => 3,
                _ => 0,
            };
            (x, y, scale, transform, false)
        } else {
            let default_scale = if name.starts_with("eDP") { 2.0 } else { 1.0 };
            (0, 0, default_scale, 0, true)
        };

        let make = item.get("make").and_then(|v| v.as_str()).unwrap_or("");
        let model = item.get("model").and_then(|v| v.as_str()).unwrap_or("");
        let serial = item.get("serial").and_then(|v| v.as_str()).unwrap_or("");

        let ident = [make, model, serial]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        let description = [make, model]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        list.push(OutputInfo {
            name: name.clone(),
            ident: if ident.is_empty() { name.clone() } else { ident },
            description: if description.is_empty() {
                name.clone()
            } else {
                description
            },
            width: active_mode.width,
            height: active_mode.height,
            refresh_mhz: active_mode.refresh_mhz,
            modes,
            x,
            y,
            scale,
            transform,
            disabled,
        });
    }

    // Sort active displays first, then by name
    list.sort_by(|a, b| a.disabled.cmp(&b.disabled).then_with(|| a.name.cmp(&b.name)));

    // Place disabled displays to the right of active displays so they don't overlap on the canvas
    let mut next_disabled_x = list
        .iter()
        .filter(|m| !m.disabled)
        .map(|m| m.x + m.logical_size().0)
        .max()
        .unwrap_or(0)
        + 160;

    for m in &mut list {
        if m.disabled {
            m.x = next_disabled_x;
            m.y = 0;
            next_disabled_x += m.logical_size().0 + 160;
        }
    }

    list
}

fn canvas_transform(outputs: &[OutputInfo], width: f32, height: f32) -> (f32, f32, f32) {
    if outputs.is_empty() {
        return (0.1, 0.0, 0.0);
    }
    let left = outputs.iter().map(|m| m.x).min().unwrap_or(0) as f32;
    let top = outputs.iter().map(|m| m.y).min().unwrap_or(0) as f32;
    let right = outputs
        .iter()
        .map(|m| m.x + m.logical_size().0)
        .max()
        .unwrap_or(1920) as f32;
    let bottom = outputs
        .iter()
        .map(|m| m.y + m.logical_size().1)
        .max()
        .unwrap_or(1080) as f32;

    let span_w = (right - left).max(1.0);
    let span_h = (bottom - top).max(1.0);

    let scale_x = (width - CANVAS_PAD * 2.0) / span_w;
    let scale_y = (height - CANVAS_PAD * 2.0) / span_h;
    let scale = scale_x.min(scale_y).clamp(0.025, 0.16);

    let ox = (width - span_w * scale) / 2.0 - left * scale;
    let oy = (height - span_h * scale) / 2.0 - top * scale;
    (scale, ox, oy)
}

pub fn build(page: &mut Page) {
    let state_guard = STATE.lock().unwrap();
    if state_guard.outputs.is_empty() {
        drop(state_guard);
        reload();
    } else {
        drop(state_guard);
    }

    let state = STATE.lock().unwrap();
    let outputs = state.outputs.clone();
    let selected = state.selected.min(outputs.len().saturating_sub(1));
    let status_text = state.status.clone();
    let mode_menu_open = state.mode_menu_open;
    drop(state);

    let theme = page.theme;
    let width = page.width;

    // 1. Visual Canvas + Monitor Tabs Card
    let canvas_widget = build_canvas(theme, width, &outputs, selected);
    page.row(CANVAS_HEIGHT, canvas_widget);

    let selector_row = build_monitor_tabs(theme, width, &outputs, selected, &status_text);
    page.row(52.0, selector_row);
    page.end_group();

    // 2. Selected Monitor Inspector Group
    if let Some(mon) = outputs.get(selected).cloned() {
        // Enable / Disable row
        let enabled = !mon.disabled;
        let enable_btn_label = if enabled { "Enabled" } else { "Disabled" };
        let enable_style = if enabled { Style::Primary } else { Style::Plain };
        let enable_control = button::view(theme, enable_btn_label, enable_style, true, move || {
            toggle_selected_enabled();
        });
        let enable_row = row::view(
            theme,
            width,
            row::HEIGHT,
            &format!("{} ({})", mon.name, mon.description),
            if enabled {
                "Active display output • Click to disable"
            } else {
                "Output is currently off • Click to enable"
            },
            enable_control,
            button::width(enable_btn_label),
        );
                page.row(row::HEIGHT, enable_row);

        // Display Brightness row
        let brightness_row = build_brightness_row(theme, width);
        page.row(row::HEIGHT, brightness_row);

        // Resolution & Refresh Rate row
        let mode_row = build_mode_row(theme, width, &mon, mode_menu_open);
        page.row(row::HEIGHT, mode_row);

        if mode_menu_open {
            let (mode_list, list_h) = build_mode_list(theme, width, &mon);
            page.row(list_h, mode_list);
        }

        // Scale Factor pills row
        let scale_row = build_scale_row(theme, width, &mon);
        page.row(row::HEIGHT, scale_row);

        // Rotation / Transform pills row
        let transform_row = build_transform_row(theme, width, &mon);
        page.row(row::HEIGHT, transform_row);

        // Position & Snap row
        let pos_row = build_position_row(theme, width, &mon);
        page.row(row::HEIGHT, pos_row);

        page.end_group();
    }

    // 3. Actions Group (Save, Preview, Reset)
    let actions_row = build_actions_row(theme, width);
    page.row(row::HEIGHT, actions_row);
    page.end_group();
}

fn build_canvas(
    theme: &Theme,
    width: f32,
    outputs: &[OutputInfo],
    selected: usize,
) -> Rectangle {
    let (scale, ox, oy) = canvas_transform(outputs, width, CANVAS_HEIGHT);

    let mut layers: Vec<Box<dyn Widget>> = Vec::new();

    // Subtle background grid lines
    let grid_color = Color::rgba(
        theme.border.red(),
        theme.border.green(),
        theme.border.blue(),
        90,
    );
    let step = (400.0 * scale).max(24.0);
    let mut gx = ox.rem_euclid(step);
    while gx < width {
        layers.push(Box::new(
            Rectangle::new()
                .width(1.0)
                .height(CANVAS_HEIGHT)
                .fill(grid_color)
                .translate(gx, 0.0),
        ));
        gx += step;
    }
    let mut gy = oy.rem_euclid(step);
    while gy < CANVAS_HEIGHT {
        layers.push(Box::new(
            Rectangle::new()
                .width(width)
                .height(1.0)
                .fill(grid_color)
                .translate(0.0, gy),
        ));
        gy += step;
    }

    // Monitor rectangles
    for (idx, mon) in outputs.iter().enumerate() {
        let (lw, lh) = mon.logical_size();
        let rx = mon.x as f32 * scale + ox;
        let ry = mon.y as f32 * scale + oy;
        let rw = (lw as f32 * scale).max(68.0);
        let rh = (lh as f32 * scale).max(46.0);

        let is_sel = idx == selected;
        let (fill, border_color, border_w) = if mon.disabled {
            (theme.background, theme.border, 1.5)
        } else if is_sel {
            (theme.selected_surface, theme.accent, 2.5)
        } else {
            (high_surface(theme), theme.border, 1.5)
        };

        let header_h = (rh * 0.28).clamp(18.0, 26.0);
        let status_label = if mon.disabled {
            format!("{} (OFF)", mon.name)
        } else {
            mon.name.clone()
        };

        let header = Rectangle::new()
            .width(rw)
            .height(header_h)
            .fill(Color::rgba(0, 0, 0, 75))
            .padding(Padding {
                top: 0.0,
                right: 6.0,
                bottom: 0.0,
                left: 8.0,
            })
            .align_child(Start, Center)
            .child(
                Text::new(&status_label)
                    .size(10.0)
                    .font(fonts::BODY)
                    .weight(Weight::SemiBold)
                    .color(if is_sel {
                        theme.accent
                    } else {
                        theme.secondary_text
                    })
                    .elide(),
            );

        let badge = Text::new(format!("{}", idx + 1))
            .size((rh * 0.34).clamp(14.0, 26.0))
            .font(fonts::BODY)
            .weight(Weight::Bold)
            .color(if mon.disabled {
                theme.muted_text
            } else if is_sel {
                theme.accent
            } else {
                theme.text
            });

        let sub_info = if mon.disabled {
            String::from("Disabled")
        } else {
            format!("{}×{} @ {}", mon.width, mon.height, fmt_scale(mon.scale))
        };

        let footer = Rectangle::new()
            .width(rw)
            .height(16.0)
            .translate(0.0, rh - 18.0)
            .align_child(Center, Center)
            .child(
                Text::new(&sub_info)
                    .size(9.0)
                    .font(fonts::NERD)
                    .color(theme.muted_text)
                    .elide(),
            );

        let center_num = Rectangle::new()
            .width(rw)
            .height(rh)
            .align_child(Center, Center)
            .child(badge);

        let card = Rectangle::new()
            .width(rw)
            .height(rh)
            .radius(10.0)
            .fill(fill)
            .border(border_w, border_color)
            .clip()
            .child(Stack::new(children![center_num, header, footer]).width(rw).height(rh))
            .translate(rx, ry);

        layers.push(Box::new(card));
    }

    // Interactive drag overlay covering the canvas
    let drag_area = Rectangle::new()
        .width(width)
        .height(CANVAS_HEIGHT)
        .cursor(Pointer)
        .on_drag(move |point| on_canvas_drag(point, width, CANVAS_HEIGHT));

    layers.push(Box::new(drag_area));

    Rectangle::new()
        .width(width)
        .height(CANVAS_HEIGHT)
        .fill(theme.surface)
        .clip()
        .child(Stack::new(layers).width(width).height(CANVAS_HEIGHT))
}

fn on_canvas_drag(point: Point, width: f32, height: f32) {
    let mut state = STATE.lock().unwrap();
    if state.outputs.is_empty() {
        return;
    }

    let now = Instant::now();
    let continuing = state
        .drag
        .as_ref()
        .is_some_and(|d| now.duration_since(d.last_tick).as_millis() < 180);

    let (scale, ox, oy) = canvas_transform(&state.outputs, width, height);

    if !continuing {
        // Hit-test from top-most to bottom-most monitor card
        let mut hit = None;
        for (idx, mon) in state.outputs.iter().enumerate().rev() {
            let (lw, lh) = mon.logical_size();
            let rx = mon.x as f32 * scale + ox;
            let ry = mon.y as f32 * scale + oy;
            let rw = (lw as f32 * scale).max(68.0);
            let rh = (lh as f32 * scale).max(46.0);
            if point.x >= rx && point.x <= rx + rw && point.y >= ry && point.y <= ry + rh {
                hit = Some((idx, point.x - rx, point.y - ry));
                break;
            }
        }

        if let Some((idx, off_x, off_y)) = hit {
            state.selected = idx;
            state.mode_menu_open = false;
            state.drag = Some(DragSession {
                index: idx,
                offset_x: off_x,
                offset_y: off_y,
                last_tick: now,
            });
        } else {
            state.drag = None;
        }
        return;
    }

    let Some(drag) = state.drag.as_mut() else {
        return;
    };
    drag.last_tick = now;
    let idx = drag.index;
    let off_x = drag.offset_x;
    let off_y = drag.offset_y;

    let new_rx = point.x - off_x;
    let new_ry = point.y - off_y;

    let mut new_x = ((new_rx - ox) / scale).round() as i32;
    let mut new_y = ((new_ry - oy) / scale).round() as i32;

    let (mw, mh) = state.outputs[idx].logical_size();
    let snap_range = (SNAP_PX / scale).round() as i32;

    // Magnetic edge and alignment snapping against other monitors
    for (other_idx, other) in state.outputs.iter().enumerate() {
        if other_idx == idx || other.disabled {
            continue;
        }
        let (ow, oh) = other.logical_size();
        let ox_m = other.x;
        let oy_m = other.y;

        if (new_x - (ox_m + ow)).abs() < snap_range {
            new_x = ox_m + ow;
        } else if ((new_x + mw) - ox_m).abs() < snap_range {
            new_x = ox_m - mw;
        } else if (new_x - ox_m).abs() < snap_range {
            new_x = ox_m;
        } else if ((new_x + mw) - (ox_m + ow)).abs() < snap_range {
            new_x = ox_m + ow - mw;
        }

        if (new_y - (oy_m + oh)).abs() < snap_range {
            new_y = oy_m + oh;
        } else if ((new_y + mh) - oy_m).abs() < snap_range {
            new_y = oy_m - mh;
        } else if (new_y - oy_m).abs() < snap_range {
            new_y = oy_m;
        } else if ((new_y + mh) - (oy_m + oh)).abs() < snap_range {
            new_y = oy_m + oh - mh;
        } else if ((new_y + mh / 2) - (oy_m + oh / 2)).abs() < snap_range {
            new_y = oy_m + oh / 2 - mh / 2;
        }
    }

    state.outputs[idx].x = new_x;
    state.outputs[idx].y = new_y;
}

fn build_monitor_tabs(
    theme: &Theme,
    width: f32,
    outputs: &[OutputInfo],
    selected: usize,
    status_text: &str,
) -> Rectangle {
    let mut tabs: Vec<Box<dyn Widget>> = Vec::new();

    for (idx, mon) in outputs.iter().enumerate() {
        let is_sel = idx == selected;
        let key = format!("mon-tab:{idx}");
        let fill = if is_sel {
            theme.accent
        } else if hovered(&key) {
            theme.hover_surface
        } else {
            high_surface(theme)
        };
        let fg = if is_sel { theme.on_accent } else { theme.text };

        let label = if mon.disabled {
            format!("{}: {} (Off)", idx + 1, mon.name)
        } else {
            format!("{}: {}", idx + 1, mon.name)
        };

        let pill_w = label.chars().count() as f32 * 7.2 + 24.0;
        let pill = Rectangle::new()
            .width(pill_w)
            .height(30.0)
            .radius(8.0)
            .fill(fill)
            .cursor(Pointer)
            .on_hover(move |inside| hover(key.clone(), inside))
            .on_click(move |_| {
                if let Ok(mut s) = STATE.lock() {
                    s.selected = idx;
                    s.mode_menu_open = false;
                }
            })
            .align_child(Center, Center)
            .child(
                Text::new(&label)
                    .size(12.0)
                    .font(fonts::BODY)
                    .weight(if is_sel { Weight::SemiBold } else { Weight::Regular })
                    .color(fg),
            );

        tabs.push(Box::new(pill));
    }

    let status = Text::new(status_text)
        .size(11.0)
        .font(fonts::BODY)
        .color(theme.muted_text)
        .elide();

    Rectangle::new()
        .width(width)
        .height(52.0)
        .padding(Padding {
            top: 0.0,
            right: 20.0,
            bottom: 0.0,
            left: 20.0,
        })
        .align_child(Start, Center)
        .child(
            Row::new(children![
                Row::new(tabs).gap(8.0).align(Center),
                status
            ])
            .width(width - 40.0)
            .justify(SpaceBetween)
            .align(Center),
        )
}

fn build_brightness_row(theme: &Theme, width: f32) -> Rectangle {
    let percent = Brightness::read().percent();
    let value = (percent as f32 / 100.0).clamp(0.01, 1.0);

    let track_w = 170.0;
    let track_h = 4.0;
    let thumb_size = 16.0;
    let val_w = 48.0;
    let gap = 8.0;

    let filled = Rectangle::new()
        .width(track_w * value)
        .height(track_h)
        .radius(Full)
        .fill(theme.accent);

    let line = Rectangle::new()
        .width(track_w)
        .height(track_h)
        .radius(Full)
        .fill(theme.border)
        .translate(0.0, (row::CONTROL_HEIGHT - track_h) / 2.0)
        .child(filled);

    let thumb_x = (track_w - thumb_size) * value;

    let thumb = Rectangle::new()
        .width(thumb_size)
        .height(thumb_size)
        .radius(Full)
        .fill(theme.accent)
        .translate(thumb_x, (row::CONTROL_HEIGHT - thumb_size) / 2.0);

    let area = Rectangle::new()
        .width(track_w)
        .height(row::CONTROL_HEIGHT)
        .cursor(Pointer)
        .on_drag(move |point| {
            let along = (point.x / track_w).clamp(0.01, 1.0);
            let new_percent = (along * 100.0).round() as u8;
            if new_percent != Brightness::read().percent() {
                Brightness::set(new_percent);
            }
        });

    let slider_stack = Stack::new(children![line, thumb, area])
        .width(track_w)
        .height(row::CONTROL_HEIGHT);

    let val_label = Text::new(format!("{percent}%"))
        .size(12.0)
        .font(fonts::BODY)
        .weight(Weight::SemiBold)
        .color(theme.secondary_text);

    let val_box = Rectangle::new()
        .width(val_w)
        .height(row::CONTROL_HEIGHT)
        .align_child(End, Center)
        .child(val_label);

    let control = Row::new(children![slider_stack, val_box])
        .gap(gap)
        .align(Center);

    let control_w = track_w + gap + val_w;

    row::view(
        theme,
        width,
        row::HEIGHT,
        "Monitor brightness",
        "Adjust display backlight level",
        control,
        control_w,
    )
}

fn build_mode_row(theme: &Theme, width: f32, mon: &OutputInfo, menu_open: bool) -> Rectangle {
    let cur_idx = mon.current_mode_index();
    let label_str = mon
        .modes
        .get(cur_idx)
        .map(Mode::label)
        .unwrap_or_else(|| format!("{} × {} @ {:.0} Hz", mon.width, mon.height, mon.refresh_mhz as f32 / 1000.0));

    let btn_label = if menu_open {
        format!("{label_str}  ▴")
    } else {
        format!("{label_str}  ▾")
    };

    let control = button::view(theme, &btn_label, Style::Plain, true, || {
        if let Ok(mut s) = STATE.lock() {
            s.mode_menu_open = !s.mode_menu_open;
        }
    });

    row::view(
        theme,
        width,
        row::HEIGHT,
        "Resolution & refresh rate",
        "Select display mode and refresh rate",
        control,
        button::width(&btn_label),
    )
}

fn build_mode_list(theme: &Theme, width: f32, mon: &OutputInfo) -> (Rectangle, f32) {
    let cur_idx = mon.current_mode_index();
    let mut rows: Vec<Box<dyn Widget>> = Vec::new();
    let content_w = width - 40.0;
    let item_w = content_w - 4.0;

    for (idx, mode) in mon.modes.iter().enumerate() {
        let is_cur = idx == cur_idx;
        let hover_key = format!("mon-mode:{idx}");
        let fill = if is_cur {
            theme.selected_surface
        } else if hovered(&hover_key) {
            theme.hover_surface
        } else {
            high_surface(theme)
        };
        let m = mode.clone();

        let mut labels: Vec<Box<dyn Widget>> = vec![Box::new(
            Text::new(mode.label())
                .size(12.0)
                .font(fonts::NERD)
                .weight(if is_cur { Weight::SemiBold } else { Weight::Regular })
                .color(if is_cur { theme.accent } else { theme.text }),
        )];

        if is_cur {
            labels.push(Box::new(
                Text::new("󰄬")
                    .size(13.0)
                    .font(fonts::NERD)
                    .color(theme.accent),
            ));
        }

        let item = Rectangle::new()
            .width(item_w)
            .height(30.0)
            .radius(8.0)
            .fill(fill)
            .padding(Padding {
                top: 0.0,
                right: 12.0,
                bottom: 0.0,
                left: 12.0,
            })
            .cursor(Pointer)
            .on_hover(move |inside| hover(hover_key.clone(), inside))
            .on_click(move |_| {
                if let Ok(mut s) = STATE.lock() {
                    let sel = s.selected;
                    if let Some(out) = s.outputs.get_mut(sel) {
                        out.width = m.width;
                        out.height = m.height;
                        out.refresh_mhz = m.refresh_mhz;
                    }
                    s.mode_menu_open = false;
                }
            })
            .align_child(Start, Center)
            .child(
                Row::new(labels)
                    .width(item_w - 24.0)
                    .justify(SpaceBetween)
                    .align(Center),
            );
        rows.push(Box::new(item));
    }

    let total_modes = mon.modes.len();
    let total_h = (total_modes as f32) * 34.0;
    let list_viewport_h = total_h.min(260.0);
    let col = Column::new(rows).gap(4.0);

    let container = Rectangle::new()
        .width(width)
        .height(list_viewport_h + 16.0)
        .padding(Padding {
            top: 8.0,
            right: 20.0,
            bottom: 8.0,
            left: 20.0,
        });

    let result = if total_h > 260.0 {
        let scroll = ScrollArea::new("mon-mode-scroll", col)
            .width(content_w)
            .height(list_viewport_h);
        container.child(scroll)
    } else {
        container.child(col)
    };

    (result, list_viewport_h + 16.0)
}

fn build_scale_row(theme: &Theme, width: f32, mon: &OutputInfo) -> Rectangle {
    let mut pills: Vec<Box<dyn Widget>> = Vec::new();

    for s in COMMON_SCALES {
        let active = (mon.scale - s).abs() < 0.02;
        let key = format!("mon-scale:{s}");
        let fill = if active {
            theme.accent
        } else if hovered(&key) {
            theme.hover_surface
        } else {
            high_surface(theme)
        };
        let fg = if active { theme.on_accent } else { theme.text };
        let label = fmt_scale(s);

        let pill = Rectangle::new()
            .width(44.0)
            .height(30.0)
            .radius(8.0)
            .fill(fill)
            .cursor(Pointer)
            .on_hover(move |inside| hover(key.clone(), inside))
            .on_click(move |_| set_selected_scale(s))
            .align_child(Center, Center)
            .child(
                Text::new(&label)
                    .size(11.5)
                    .font(fonts::BODY)
                    .weight(if active { Weight::SemiBold } else { Weight::Regular })
                    .color(fg),
            );
        pills.push(Box::new(pill));
    }

    let control_w = COMMON_SCALES.len() as f32 * 44.0 + (COMMON_SCALES.len() - 1) as f32 * 6.0;
    let control = Row::new(pills).gap(6.0).align(Center);

    row::view(
        theme,
        width,
        row::HEIGHT,
        "Scale factor",
        "UI scaling multiplier for the selected display",
        control,
        control_w,
    )
}

fn build_transform_row(theme: &Theme, width: f32, mon: &OutputInfo) -> Rectangle {
    let mut pills: Vec<Box<dyn Widget>> = Vec::new();

    for (label, code) in TRANSFORMS {
        let active = mon.transform == code;
        let key = format!("mon-trans:{code}");
        let fill = if active {
            theme.accent
        } else if hovered(&key) {
            theme.hover_surface
        } else {
            high_surface(theme)
        };
        let fg = if active { theme.on_accent } else { theme.text };

        let pill = Rectangle::new()
            .width(62.0)
            .height(30.0)
            .radius(8.0)
            .fill(fill)
            .cursor(Pointer)
            .on_hover(move |inside| hover(key.clone(), inside))
            .on_click(move |_| {
                if let Ok(mut s) = STATE.lock() {
                    let sel = s.selected;
                    if let Some(out) = s.outputs.get_mut(sel) {
                        out.transform = code;
                    }
                }
            })
            .align_child(Center, Center)
            .child(
                Text::new(label)
                    .size(11.5)
                    .font(fonts::BODY)
                    .weight(if active { Weight::SemiBold } else { Weight::Regular })
                    .color(fg),
            );
        pills.push(Box::new(pill));
    }

    let control_w = TRANSFORMS.len() as f32 * 62.0 + (TRANSFORMS.len() - 1) as f32 * 6.0;
    let control = Row::new(pills).gap(6.0).align(Center);

    row::view(
        theme,
        width,
        row::HEIGHT,
        "Orientation",
        "Display rotation transform",
        control,
        control_w,
    )
}

fn build_position_row(theme: &Theme, width: f32, mon: &OutputInfo) -> Rectangle {
    let snap_left = button::view(theme, "Snap Left", Style::Plain, true, || snap_selected(-1));
    let snap_right = button::view(theme, "Snap Right", Style::Plain, true, || snap_selected(1));
    let zero_btn = button::view(theme, "Origin (0, 0)", Style::Plain, true, || {
        if let Ok(mut s) = STATE.lock() {
            let sel = s.selected;
            if let Some(out) = s.outputs.get_mut(sel) {
                out.x = 0;
                out.y = 0;
            }
        }
    });

    let w_left = button::width("Snap Left");
    let w_right = button::width("Snap Right");
    let w_zero = button::width("Origin (0, 0)");
    let total_w = w_left + w_right + w_zero + 16.0;

    let control = Row::new(children![snap_left, snap_right, zero_btn])
        .gap(8.0)
        .align(Center);

    row::view(
        theme,
        width,
        row::HEIGHT,
        &format!("Position: X = {}, Y = {}", mon.x, mon.y),
        "Drag on the canvas above or snap adjacent to active displays",
        control,
        total_w,
    )
}

fn build_actions_row(theme: &Theme, width: f32) -> Rectangle {
    let save_btn = button::view(theme, "Save Layout", Style::Primary, true, save_layout);
    let preview_btn = button::view(theme, "Preview", Style::Plain, true, preview_layout);
    let reset_btn = button::view(theme, "Reset", Style::Plain, true, reload);

    let total_w =
        button::width("Save Layout") + button::width("Preview") + button::width("Reset") + 16.0;

    let control = Row::new(children![reset_btn, preview_btn, save_btn])
        .gap(8.0)
        .align(Center);

    row::view(
        theme,
        width,
        row::HEIGHT,
        "Apply display layout",
        "Write output rules to Niri config and migrate windows safely",
        control,
        total_w,
    )
}

fn toggle_selected_enabled() {
    let Ok(mut state) = STATE.lock() else {
        return;
    };
    let sel = state.selected;
    let Some(cur_disabled) = state.outputs.get(sel).map(|m| m.disabled) else {
        return;
    };

    if !cur_disabled {
        // Disabling: ensure at least 1 active monitor remains
        let active_count = state.outputs.iter().filter(|m| !m.disabled).count();
        if active_count <= 1 {
            state.status = String::from("Safety guard: Cannot disable the last active display!");
            return;
        }
        state.outputs[sel].disabled = true;
        state.status = format!("Marked {} as disabled (click Save Layout to apply).", state.outputs[sel].name);
    } else {
        let max_right = state
            .outputs
            .iter()
            .enumerate()
            .filter(|(i, m)| *i != sel && !m.disabled)
            .map(|(_, m)| m.x + m.logical_size().0)
            .max()
            .unwrap_or(0);
        state.outputs[sel].disabled = false;
        state.outputs[sel].x = max_right;
        state.outputs[sel].y = 0;
        state.status = format!("Enabled {} at X = {}, Y = 0.", state.outputs[sel].name, max_right);
    }
}

fn set_selected_scale(new_scale: f32) {
    let Ok(mut state) = STATE.lock() else {
        return;
    };
    let sel = state.selected;
    if sel >= state.outputs.len() {
        return;
    }

    let (old_w, _) = state.outputs[sel].logical_size();
    let old_x = state.outputs[sel].x;
    state.outputs[sel].scale = new_scale;
    let (new_w, _) = state.outputs[sel].logical_size();
    let delta_w = new_w - old_w;
    let right_edge = old_x + old_w;

    for (idx, other) in state.outputs.iter_mut().enumerate() {
        if idx != sel && !other.disabled && (other.x - right_edge).abs() < 25 {
            other.x += delta_w;
        }
    }
}

fn snap_selected(direction: i32) {
    let Ok(mut state) = STATE.lock() else {
        return;
    };
    let sel = state.selected;
    if sel >= state.outputs.len() {
        return;
    }
    let (mw, _) = state.outputs[sel].logical_size();

    if direction < 0 {
        let min_left = state
            .outputs
            .iter()
            .enumerate()
            .filter(|(i, m)| *i != sel && !m.disabled)
            .map(|(_, m)| m.x)
            .min()
            .unwrap_or(0);
        state.outputs[sel].x = min_left - mw;
        state.outputs[sel].y = 0;
    } else {
        let max_right = state
            .outputs
            .iter()
            .enumerate()
            .filter(|(i, m)| *i != sel && !m.disabled)
            .map(|(_, m)| m.x + m.logical_size().0)
            .max()
            .unwrap_or(0);
        state.outputs[sel].x = max_right;
        state.outputs[sel].y = 0;
    }
}

fn normalize_outputs(outputs: &mut [OutputInfo]) {
    let min_x = outputs
        .iter()
        .filter(|m| !m.disabled)
        .map(|m| m.x)
        .min()
        .unwrap_or(0);
    let min_y = outputs
        .iter()
        .filter(|m| !m.disabled)
        .map(|m| m.y)
        .min()
        .unwrap_or(0);

    for m in outputs.iter_mut().filter(|m| !m.disabled) {
        m.x -= min_x;
        m.y -= min_y;
    }
}

fn apply_live(outputs: &[OutputInfo]) {
    for m in outputs {
        if m.disabled {
            let _ = Command::new("niri")
                .args(["msg", "output", &m.name, "off"])
                .output();
            continue;
        }
        let mode_arg = format!(
            "{}x{}@{:.3}",
            m.width,
            m.height,
            m.refresh_mhz as f32 / 1000.0
        );
        let scale_arg = format!("{:.2}", m.scale);
        let trans_arg = match m.transform {
            1 => "90",
            2 => "180",
            3 => "270",
            _ => "normal",
        };

        let _ = Command::new("niri")
            .args(["msg", "output", &m.name, "on"])
            .output();
        let _ = Command::new("niri")
            .args(["msg", "output", &m.name, "mode", &mode_arg])
            .output();
        let _ = Command::new("niri")
            .args(["msg", "output", &m.name, "scale", &scale_arg])
            .output();
        let _ = Command::new("niri")
            .args(["msg", "output", &m.name, "transform", trans_arg])
            .output();
        let _ = Command::new("niri")
            .args([
                "msg",
                "output",
                &m.name,
                "position",
                "set",
                &m.x.to_string(),
                &m.y.to_string(),
            ])
            .output();
    }

    crate::monitors::sync_and_guard();
}

fn preview_layout() {
    let outputs = {
        let Ok(mut state) = STATE.lock() else {
            return;
        };
        normalize_outputs(&mut state.outputs);
        state.status = String::from("Live preview applied to Niri.");
        state.outputs.clone()
    };
    apply_live(&outputs);
}

fn save_layout() {
    let outputs = {
        let Ok(mut state) = STATE.lock() else {
            return;
        };
        normalize_outputs(&mut state.outputs);
        state.status = String::from("Layout saved to amane.kdl and applied!");
        state.outputs.clone()
    };

    write_niri_outputs(&outputs);
    apply_live(&outputs);
}

fn write_niri_outputs(outputs: &[OutputInfo]) {
    let home = std::env::var("HOME").unwrap_or_default();
    let path = PathBuf::from(format!("{home}/.config/niri/amane.kdl"));
    let existing = fs::read_to_string(&path).unwrap_or_default();

    // Strip any existing `output "..." { ... }` blocks and `// Display & Monitor Layouts` comments
    let mut cleaned_lines: Vec<&str> = Vec::new();
    let mut inside_output = false;
    let mut brace_depth: i32 = 0;

    for line in existing.lines() {
        let trimmed = line.trim();
        if trimmed == "// Display & Monitor Layouts" {
            continue;
        }
        if !inside_output && trimmed.starts_with("output ") && trimmed.contains('{') {
            inside_output = true;
            brace_depth = line.chars().filter(|&c| c == '{').count() as i32
                - line.chars().filter(|&c| c == '}').count() as i32;
            if brace_depth <= 0 {
                inside_output = false;
            }
            continue;
        }
        if inside_output {
            brace_depth += line.chars().filter(|&c| c == '{').count() as i32;
            brace_depth -= line.chars().filter(|&c| c == '}').count() as i32;
            if brace_depth <= 0 {
                inside_output = false;
            }
            continue;
        }
        cleaned_lines.push(line);
    }

    let mut out = cleaned_lines.join("\n").trim_end().to_string();
    out.push_str("\n\n// Display & Monitor Layouts\n");

    for m in outputs {
        let target_name = if m.name.starts_with("eDP") {
            &m.name
        } else {
            &m.ident
        };

        if m.disabled {
            out.push_str(&format!("output \"{target_name}\" {{\n    off\n}}\n"));
        } else {
            let trans = match m.transform {
                1 => "90",
                2 => "180",
                3 => "270",
                _ => "normal",
            };
            out.push_str(&format!(
                "output \"{target_name}\" {{\n    mode \"{}x{}@{:.3}\"\n    position x={} y={}\n    scale {:.2}\n    transform \"{trans}\"\n}}\n",
                m.width,
                m.height,
                m.refresh_mhz as f32 / 1000.0,
                m.x,
                m.y,
                m.scale
            ));
        }
    }

    let _ = fs::write(&path, out);
}
