use std::fs;
use std::process::Command;

use amane::{
    Center, Color, Column, Cpu, Memory, Padding, Pointer, Rectangle, Row,
    Service, SpaceBetween, Start, Text, Weight, Widget, children,
};

use super::{Overlay, PanelView, Region};
use crate::floating::sensors::Sensors;
use crate::fonts;
use crate::theme::Theme;

const CARD_PADDING: f32 = 16.0;
const CARD_RADIUS: f32 = 20.0;

#[derive(Clone, Default)]
pub struct ProcessInfo {
    pub name: String,
    pub mem_mb: f32,
}

#[derive(Clone, Default)]
pub struct SysmonData {
    pub swap_used_gib: f32,
    pub swap_total_gib: f32,
    pub load1: String,
    pub load5: String,
    pub load15: String,
    pub top_procs: Vec<ProcessInfo>,
}

impl Service for SysmonData {
    fn new() -> Self {
        let mut data = Self::default();
        data.update();
        data
    }

    fn interval() -> std::time::Duration {
        std::time::Duration::from_secs(2)
    }

    fn update(&mut self) -> bool {
        let (swap_used, swap_total) = read_swap();
        self.swap_used_gib = swap_used;
        self.swap_total_gib = swap_total;

        let (l1, l5, l15) = read_loadavg();
        self.load1 = l1;
        self.load5 = l5;
        self.load15 = l15;

        self.top_procs = read_top_processes(5);
        true
    }
}

#[allow(dead_code)]
pub fn view(_overlay: &Overlay, _theme: &Theme, _screen: Region) -> Option<PanelView> {
    None
}

// Two-column layout rendered inside the middle pulldown's System Monitor tab
pub fn embedded_view(theme: &Theme, width: f32, height: f32) -> Row {
    let data = SysmonData::read();

    let gap = 14.0;
    let col_width = (width - gap) / 2.0;
    let v_gap = 12.0;
    let half_height = (height - v_gap) / 2.0;

    let left_col = Column::new(children![
        memory_card(theme, &data, col_width, half_height),
        cpu_card(theme, &data, col_width, half_height),
    ])
    .gap(v_gap)
    .width(col_width);

    let right_col = processes_card(theme, &data, col_width, height);

    Row::new(children![left_col, right_col])
        .width(width)
        .height(height)
        .gap(gap)
        .align(Center)
}

fn memory_card(theme: &Theme, data: &SysmonData, width: f32, height: f32) -> Rectangle {
    let inner_width = width - CARD_PADDING * 2.0;

    let memory = Memory::read();
    let total_gib = memory.total_kib() as f32 / 1024.0 / 1024.0;
    let used_gib = memory.used_kib() as f32 / 1024.0 / 1024.0;
    let percent = memory.percent();
    let usage = (percent as f32 / 100.0).clamp(0.0, 1.0);

    let bar_color = if percent < 60 {
        theme.success
    } else if percent < 85 {
        theme.accent
    } else {
        theme.danger
    };

    let icon = Text::new("󰍛")
        .size(15.0)
        .font(fonts::NERD)
        .tight()
        .color(theme.accent);

    let title = Text::new("Memory (RAM)")
        .size(13.0)
        .font(fonts::BODY)
        .weight(Weight::SemiBold)
        .color(theme.text);

    let left_title = Row::new(children![icon, title]).gap(6.0).align(Center);

    let stat = Text::new(format!("{used_gib:.1} / {total_gib:.1} GiB ({percent}%)"))
        .size(12.0)
        .font(fonts::BODY)
        .weight(Weight::Medium)
        .color(theme.secondary_text);

    let top_row = Row::new(children![left_title, stat])
        .width(inner_width)
        .justify(SpaceBetween)
        .align(Center);

    let bar = progress_bar(theme, usage, bar_color, inner_width);

    let swap_text = if data.swap_total_gib > 0.0 {
        format!("Swap: {:.1} / {:.1} GiB", data.swap_used_gib, data.swap_total_gib)
    } else {
        String::from("Swap: Off")
    };

    let free_gib = (total_gib - used_gib).max(0.0);
    let sub_left = Text::new(format!("Free: {free_gib:.1} GiB"))
        .size(11.0)
        .font(fonts::BODY)
        .color(theme.secondary_text);

    let sub_right = Text::new(swap_text)
        .size(11.0)
        .font(fonts::BODY)
        .color(theme.secondary_text);

    let sub_row = Row::new(children![sub_left, sub_right])
        .width(inner_width)
        .justify(SpaceBetween)
        .align(Center);

    let card_content = Column::new(children![top_row, bar, sub_row])
        .gap(12.0)
        .width(inner_width);

    Rectangle::new()
        .width(width)
        .height(height)
        .radius(CARD_RADIUS)
        .fill(theme.surface)
        .padding(Padding {
            top: CARD_PADDING,
            right: CARD_PADDING,
            bottom: CARD_PADDING,
            left: CARD_PADDING,
        })
        .child(card_content)
}

fn cpu_card(theme: &Theme, data: &SysmonData, width: f32, height: f32) -> Rectangle {
    let inner_width = width - CARD_PADDING * 2.0;

    let cpu = Cpu::read();
    let percent = cpu.percent();
    let usage = (percent as f32 / 100.0).clamp(0.0, 1.0);

    let bar_color = if percent < 50 {
        theme.success
    } else if percent < 80 {
        theme.accent
    } else {
        theme.danger
    };

    let icon = Text::new("󰻠")
        .size(15.0)
        .font(fonts::NERD)
        .tight()
        .color(theme.accent);

    let title = Text::new("CPU")
        .size(13.0)
        .font(fonts::BODY)
        .weight(Weight::SemiBold)
        .color(theme.text);

    let left_title = Row::new(children![icon, title]).gap(6.0).align(Center);

    let temp_str = Sensors::read()
        .cpu
        .map(|t| format!(" • {t}°C"))
        .unwrap_or_default();

    let stat = Text::new(format!("{percent}%{temp_str}"))
        .size(12.0)
        .font(fonts::BODY)
        .weight(Weight::Medium)
        .color(theme.secondary_text);

    let top_row = Row::new(children![left_title, stat])
        .width(inner_width)
        .justify(SpaceBetween)
        .align(Center);

    let bar = progress_bar(theme, usage, bar_color, inner_width);

    let load_text = Text::new(format!("Load avg: {}  {}  {}", data.load1, data.load5, data.load15))
        .size(11.0)
        .font(fonts::BODY)
        .color(theme.secondary_text);

    let card_content = Column::new(children![top_row, bar, load_text])
        .gap(12.0)
        .width(inner_width);

    Rectangle::new()
        .width(width)
        .height(height)
        .radius(CARD_RADIUS)
        .fill(theme.surface)
        .padding(Padding {
            top: CARD_PADDING,
            right: CARD_PADDING,
            bottom: CARD_PADDING,
            left: CARD_PADDING,
        })
        .child(card_content)
}

fn processes_card(theme: &Theme, data: &SysmonData, width: f32, height: f32) -> Rectangle {
    let inner_width = width - CARD_PADDING * 2.0;

    let title = Text::new("Top Processes (Memory)")
        .size(13.0)
        .font(fonts::BODY)
        .weight(Weight::SemiBold)
        .color(theme.text);

    let term_icon = Text::new("󰆍")
        .size(13.0)
        .font(fonts::NERD)
        .color(theme.secondary_text);

    let term_text = Text::new("Top")
        .size(11.0)
        .font(fonts::BODY)
        .weight(Weight::Medium)
        .color(theme.secondary_text);

    let term_button = Rectangle::new()
        .width(54.0)
        .height(24.0)
        .radius(8.0)
        .fill(theme.selected_surface)
        .align_child(Center, Center)
        .cursor(Pointer)
        .on_click(|_| {
            let _ = Command::new("alacritty").arg("-e").arg("top").spawn();
        })
        .child(Row::new(children![term_icon, term_text]).gap(4.0).align(Center));

    let header = Row::new(children![title, term_button])
        .width(inner_width)
        .justify(SpaceBetween)
        .align(Center);

    let mut rows: Vec<Box<dyn Widget>> = vec![Box::new(header)];

    for p in &data.top_procs {
        let name_text = Text::new(&p.name)
            .size(12.0)
            .font(fonts::BODY)
            .weight(Weight::Medium)
            .color(theme.text)
            .elide();

        let name_box = Rectangle::new()
            .width(inner_width - 80.0)
            .height(22.0)
            .align_child(Start, Center)
            .child(name_text);

        let mem_text = Text::new(format!("{:.0} MB", p.mem_mb))
            .size(11.0)
            .font(fonts::BODY)
            .weight(Weight::SemiBold)
            .color(theme.accent);

        let row = Row::new(children![name_box, mem_text])
            .width(inner_width)
            .justify(SpaceBetween)
            .align(Center);

        rows.push(Box::new(row));
    }

    let card_content = Column::new(rows).gap(10.0).width(inner_width);

    Rectangle::new()
        .width(width)
        .height(height)
        .radius(CARD_RADIUS)
        .fill(theme.surface)
        .padding(Padding {
            top: CARD_PADDING,
            right: CARD_PADDING,
            bottom: CARD_PADDING,
            left: CARD_PADDING,
        })
        .child(card_content)
}

fn progress_bar(theme: &Theme, amount: f32, color: Color, width: f32) -> Rectangle {
    const BAR_HEIGHT: f32 = 8.0;

    let fill_width = (width * amount).max(BAR_HEIGHT);

    let fill = Rectangle::new()
        .width(fill_width)
        .height(BAR_HEIGHT)
        .radius(BAR_HEIGHT / 2.0)
        .fill(color);

    Rectangle::new()
        .width(width)
        .height(BAR_HEIGHT)
        .radius(BAR_HEIGHT / 2.0)
        .fill(theme.border)
        .child(fill)
}

fn read_swap() -> (f32, f32) {
    let Ok(text) = fs::read_to_string("/proc/meminfo") else {
        return (0.0, 0.0);
    };

    let total = parse_kb(&text, "SwapTotal:");
    let free = parse_kb(&text, "SwapFree:");
    let used = total.saturating_sub(free);

    (used as f32 / 1024.0 / 1024.0, total as f32 / 1024.0 / 1024.0)
}

fn parse_kb(text: &str, prefix: &str) -> u64 {
    for line in text.lines() {
        let Some(rest) = line.strip_prefix(prefix) else {
            continue;
        };
        return rest.split_whitespace().next().and_then(|s| s.parse().ok()).unwrap_or(0);
    }
    0
}

fn read_loadavg() -> (String, String, String) {
    let Ok(text) = fs::read_to_string("/proc/loadavg") else {
        return ("0.00".into(), "0.00".into(), "0.00".into());
    };

    let mut parts = text.split_whitespace();
    (
        parts.next().unwrap_or("0.00").to_string(),
        parts.next().unwrap_or("0.00").to_string(),
        parts.next().unwrap_or("0.00").to_string(),
    )
}

fn read_top_processes(limit: usize) -> Vec<ProcessInfo> {
    let Ok(entries) = fs::read_dir("/proc") else {
        return Vec::new();
    };

    let mut procs = Vec::new();

    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let Some(name_str) = file_name.to_str() else {
            continue;
        };

        if !name_str.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }

        let path = entry.path();
        let Ok(statm) = fs::read_to_string(path.join("statm")) else {
            continue;
        };

        let rss_pages: u64 = statm
            .split_whitespace()
            .nth(1)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let mem_kb = rss_pages * 4;
        let mem_mb = mem_kb as f32 / 1024.0;

        if mem_mb < 10.0 {
            continue;
        }

        let comm = fs::read_to_string(path.join("comm"))
            .unwrap_or_else(|_| String::from("unknown"))
            .trim()
            .to_string();

        procs.push(ProcessInfo {
            name: comm,
            mem_mb,
        });
    }

    procs.sort_by(|a, b| b.mem_mb.partial_cmp(&a.mem_mb).unwrap_or(std::cmp::Ordering::Equal));
    procs.truncate(limit);
    procs
}

pub fn ipc(_arguments: &[String]) -> String {
    Overlay::toggle_sysmon();
    String::from("ok")
}
