use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::OnceLock;

use amane::{
    Center, Column, End, Image, Padding, Rectangle, Row, Service, Start, Text, TextInput,
    Weight, Widget, children,
};

use super::super::button::{self, Style};
use super::super::field;
use super::super::page::Page;
use crate::fonts;
use crate::profile::Profile;
use crate::theme::Theme;

const AVATAR: f32 = 72.0;
const AVATAR_ICON: &str = "󰀄";

const PROFILE_HEIGHT: f32 = 128.0;

// the text input's name, which keeps what was typed between redraws
pub const NAME_INPUT: &str = "profile-name";

struct StaticSysInfo {
    distro: String,
    logo_path: Option<String>,
    kernel: String,
    packages: String,
    cpu: String,
    memory: String,
    gpu: String,
}

static STATIC_INFO: OnceLock<StaticSysInfo> = OnceLock::new();

fn get_static_info() -> &'static StaticSysInfo {
    STATIC_INFO.get_or_init(|| {
        let mut distro = String::from("Linux");
        for p in &["/etc/os-release", "/usr/lib/os-release"] {
            if let Ok(c) = fs::read_to_string(p) {
                for line in c.lines() {
                    if let Some(val) = line.strip_prefix("PRETTY_NAME=") {
                        distro = val.trim_matches('"').trim_matches('\'').to_string();
                        break;
                    }
                }
                if distro != "Linux" {
                    break;
                }
            }
        }

        let mut logo_path = None;
        let candidates = [
            "/usr/share/pixmaps/fedora-logo.png",
            "/usr/share/icons/hicolor/256x256/apps/fedora-logo-icon.png",
            "/usr/share/pixmaps/fedora_logo_med.png",
        ];
        for c in candidates {
            if Path::new(c).exists() {
                logo_path = Some(c.to_string());
                break;
            }
        }

        let kernel = fs::read_to_string("/proc/sys/kernel/osrelease")
            .unwrap_or_else(|_| String::from("Unknown"))
            .trim()
            .to_string();

        let packages = Command::new("sh")
            .args(["-c", "rpm -qa | wc -l"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| format!("{} (rpm)", s.trim()))
            .unwrap_or_else(|| String::from("Unknown"));

        let mut cpu = String::from("Unknown");
        if let Ok(c) = fs::read_to_string("/proc/cpuinfo") {
            for line in c.lines() {
                if line.contains("model name") {
                    if let Some((_, val)) = line.split_once(':') {
                        let clean = val.replace("(R)", "").replace("(TM)", "").replace(" CPU", "");
                        cpu = clean.trim().to_string();
                        break;
                    }
                }
            }
        }

        let mut memory = String::from("Unknown");
        if let Ok(c) = fs::read_to_string("/proc/meminfo") {
            for line in c.lines() {
                if line.starts_with("MemTotal:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(kb) = parts[1].parse::<f64>() {
                            memory = format!("{:.1} GiB", kb / 1024.0 / 1024.0);
                        }
                    }
                    break;
                }
            }
        }

        let mut gpu = String::from("Integrated Graphics");
        if let Ok(o) = Command::new("lspci").output() {
            if let Ok(s) = String::from_utf8(o.stdout) {
                for line in s.lines() {
                    let lower = line.to_lowercase();
                    if lower.contains("vga compatible controller") || lower.contains("3d controller") {
                        if let Some((_, rest)) = line.split_once(": ") {
                            let clean = rest.replace("Corporation ", "").replace("Intel ", "");
                            gpu = clean.trim().to_string();
                            break;
                        }
                    }
                }
            }
        }

        StaticSysInfo {
            distro,
            logo_path,
            kernel,
            packages,
            cpu,
            memory,
            gpu,
        }
    })
}

fn get_uptime() -> String {
    if let Ok(c) = fs::read_to_string("/proc/uptime") {
        if let Some(first) = c.split_whitespace().next() {
            if let Ok(secs) = first.parse::<f64>() {
                let s = secs as u64;
                let days = s / 86400;
                let hours = (s % 86400) / 3600;
                let mins = (s % 3600) / 60;
                if days > 0 {
                    return format!("{days}d {hours}h {mins}m");
                } else if hours > 0 {
                    return format!("{hours}h {mins}m");
                } else {
                    return format!("{mins}m");
                }
            }
        }
    }
    String::from("Unknown")
}

// System specs & info, followed by profile details
pub fn build(page: &mut Page) {
    let info = get_static_info();
    let uptime = get_uptime();

    // Group 1: System Information Hero Card & Specs
    page.row(112.0, hero_card(page, info, &uptime));
    page.row(48.0, spec_row(page.theme, page.width, "Operating System", &info.distro));
    page.row(48.0, spec_row(page.theme, page.width, "Kernel Release", &info.kernel));
    page.row(48.0, spec_row(page.theme, page.width, "System Uptime", &uptime));
    page.row(48.0, spec_row(page.theme, page.width, "Installed Packages", &info.packages));
    page.row(48.0, spec_row(page.theme, page.width, "Processor", &info.cpu));
    page.row(48.0, spec_row(page.theme, page.width, "Total Memory", &info.memory));
    page.row(48.0, spec_row(page.theme, page.width, "Graphics Adapter", &info.gpu));
    page.end_group();

    // Group 2: User Profile
    let profile = profile(page);
    page.row(PROFILE_HEIGHT, profile);

    let login = env::var("USER").unwrap_or_default();
    let input = TextInput::new(NAME_INPUT)
        .size(14.0)
        .color(page.theme.text)
        .placeholder(&login)
        .on_change(Profile::set_name);

    let name = field::view(
        page,
        "Display name",
        "Name shown on the lock screen; this does not change your system account",
        input,
    );

    page.row(super::super::row::TALL_HEIGHT, name);
    page.end_group();
}

fn hero_card(page: &Page, info: &StaticSysInfo, uptime: &str) -> Rectangle {
    let theme = page.theme;

    let logo_box = Rectangle::new()
        .width(72.0)
        .height(72.0)
        .radius(16.0)
        .fill(theme.selected_surface)
        .clip();

    let logo = if let Some(ref path) = info.logo_path {
        if Image::loaded(path) {
            logo_box.fill(Image::cover(path).thumbnail(144, 144))
        } else {
            logo_box.align_child(Center, Center).child(
                Text::new("\u{f0379}")
                    .size(34.0)
                    .font(fonts::NERD)
                    .color(theme.accent),
            )
        }
    } else {
        logo_box.align_child(Center, Center).child(
            Text::new("\u{f0379}")
                .size(34.0)
                .font(fonts::NERD)
                .color(theme.accent),
        )
    };

    let title = Text::new(&info.distro)
        .size(19.0)
        .font(fonts::BODY)
        .weight(Weight::Bold)
        .color(theme.text);

    let subtitle = Text::new(format!("Kernel {}  •  Uptime {}", info.kernel, uptime))
        .size(12.0)
        .font(fonts::BODY)
        .weight(Weight::SemiBold)
        .color(theme.accent);

    let cpu_label = Text::new(&info.cpu)
        .size(12.0)
        .font(fonts::BODY)
        .color(theme.muted_text)
        .elide();

    let details = Column::new(children![title, subtitle, cpu_label]).gap(4.0);

    let labels_width = page.width - 48.0 - 72.0 - 16.0;

    let details = Rectangle::new()
        .width(labels_width)
        .height(72.0)
        .align_child(Start, Center)
        .child(details);

    let content = Row::new(children![logo, details]).gap(16.0).align(Center);

    Rectangle::new()
        .width(page.width)
        .height(112.0)
        .padding(Padding {
            top: 0.0,
            right: 24.0,
            bottom: 0.0,
            left: 24.0,
        })
        .align_child(Start, Center)
        .child(content)
}

fn spec_row(theme: &Theme, width: f32, label: &str, value: &str) -> Rectangle {
    let content_width = width - 48.0;
    let label_width = 180.0;
    let value_width = content_width - label_width;

    let label_widget = Text::new(label)
        .size(14.0)
        .font(fonts::BODY)
        .weight(Weight::Medium)
        .color(theme.text);

    let label_box = Rectangle::new()
        .width(label_width)
        .align_child(Start, Center)
        .child(label_widget);

    let value_widget = Text::new(value)
        .size(13.0)
        .font(fonts::BODY)
        .color(theme.secondary_text)
        .elide();

    let value_box = Rectangle::new()
        .width(value_width)
        .align_child(End, Center)
        .child(value_widget);

    let row = Row::new(children![label_box, value_box])
        .width(content_width)
        .align(Center);

    Rectangle::new()
        .width(width)
        .height(48.0)
        .padding(Padding {
            top: 0.0,
            right: 24.0,
            bottom: 0.0,
            left: 24.0,
        })
        .align_child(Start, Center)
        .child(row)
}

fn profile(page: &Page) -> Rectangle {
    let theme = page.theme;

    let profile = Profile::read();

    let login = env::var("USER").unwrap_or_default();

    let has_picture = profile.picture().is_some();

    let labels = Column::new(children![
        Text::new(profile.name())
            .size(17.0)
            .font(fonts::BODY)
            .weight(Weight::Bold)
            .color(theme.text)
            .elide(),
        Text::new(&login)
            .size(12.0)
            .font(fonts::BODY)
            .color(theme.muted_text)
            .elide(),
    ])
    .gap(2.0);

    let mut buttons: Vec<Box<dyn Widget>> = Vec::new();

    let mut buttons_width = button::width("Change photo");

    if has_picture {
        buttons.push(Box::new(button::view(
            theme,
            "Default",
            Style::Danger,
            true,
            Profile::reset_picture,
        )));

        buttons_width += button::width("Default") + 8.0;
    }

    buttons.push(Box::new(button::view(
        theme,
        "Change photo",
        Style::Plain,
        true,
        Profile::pick_picture,
    )));

    let labels_width = page.width - 40.0 - AVATAR - 32.0 - buttons_width;

    let labels = Rectangle::new()
        .width(labels_width)
        .height(44.0)
        .align_child(Start, Center)
        .child(labels);

    let row = Row::new(children![avatar(&profile, page), labels, Row::new(buttons).gap(8.0)])
        .gap(16.0)
        .align(Center);

    Rectangle::new()
        .width(page.width)
        .height(PROFILE_HEIGHT)
        .padding(Padding {
            top: 0.0,
            right: 20.0,
            bottom: 0.0,
            left: 20.0,
        })
        .align_child(Start, Center)
        .child(row)
}

fn avatar(profile: &Profile, page: &Page) -> Rectangle {
    let theme = page.theme;

    let avatar = Rectangle::new()
        .width(AVATAR)
        .height(AVATAR)
        .radius(24.0)
        .fill(theme.selected_surface)
        .clip();

    if let Some(picture) = profile.picture() {
        if Image::loaded(&picture) {
            return avatar.fill(Image::cover(picture).thumbnail(144, 144));
        }
    }

    avatar.align_child(Center, Center).child(
        Text::new(AVATAR_ICON)
            .size(34.0)
            .font(fonts::NERD)
            .color(theme.accent),
    )
}
