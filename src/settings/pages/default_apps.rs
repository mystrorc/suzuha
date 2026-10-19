use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{LazyLock, Mutex};

use amane::{Center, Column, Padding, Pointer, Rectangle, Row, Start, Text, Weight, children};

use super::super::button::{self, Style};
use super::super::page::Page;
use super::super::{high_surface, row};
use crate::fonts;
use crate::theme::Theme;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvailableApp {
    pub desktop_id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct DefaultAppsState {
    pub browsers: Vec<AvailableApp>,
    pub file_managers: Vec<AvailableApp>,
    pub terminals: Vec<AvailableApp>,
    pub editors: Vec<AvailableApp>,

    pub current_browser_idx: usize,
    pub current_fm_idx: usize,
    pub current_term_idx: usize,
    pub current_editor_idx: usize,

    pub status_message: String,
}

impl Default for DefaultAppsState {
    fn default() -> Self {
        let (browsers, file_managers, terminals, editors) = scan_available_apps();

        let cur_browser = get_current_browser();
        let cur_fm = get_current_fm();
        let cur_term = get_current_term();
        let cur_editor = get_current_editor();

        let current_browser_idx = browsers
            .iter()
            .position(|a| a.desktop_id == cur_browser || cur_browser.contains(&a.desktop_id))
            .unwrap_or(0);
        let current_fm_idx = file_managers
            .iter()
            .position(|a| a.desktop_id == cur_fm || cur_fm.contains(&a.desktop_id))
            .unwrap_or(0);
        let current_term_idx = terminals
            .iter()
            .position(|a| a.desktop_id == cur_term || cur_term.contains(&a.desktop_id))
            .unwrap_or(0);
        let current_editor_idx = editors
            .iter()
            .position(|a| a.desktop_id == cur_editor || cur_editor.contains(&a.desktop_id))
            .unwrap_or(0);

        Self {
            browsers,
            file_managers,
            terminals,
            editors,
            current_browser_idx,
            current_fm_idx,
            current_term_idx,
            current_editor_idx,
            status_message: "Configure preferred applications managed via XDG specifications.".to_string(),
        }
    }
}

pub static STATE: LazyLock<Mutex<DefaultAppsState>> = LazyLock::new(|| Mutex::new(DefaultAppsState::default()));

pub fn scan_available_apps() -> (Vec<AvailableApp>, Vec<AvailableApp>, Vec<AvailableApp>, Vec<AvailableApp>) {
    let mut search_dirs = Vec::new();
    if let Some(home) = std::env::var_os("HOME") {
        let h = PathBuf::from(home);
        search_dirs.push(h.join(".local/share/applications"));
        search_dirs.push(h.join(".local/share/flatpak/exports/share/applications"));
    }
    search_dirs.push(PathBuf::from("/usr/local/share/applications"));
    search_dirs.push(PathBuf::from("/usr/share/applications"));
    search_dirs.push(PathBuf::from("/var/lib/flatpak/exports/share/applications"));

    let mut apps: HashMap<String, (String, Vec<String>, Vec<String>)> = HashMap::new();

    for dir in search_dirs {
        if !dir.is_dir() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("desktop") {
                    continue;
                }
                let desktop_id = match path.file_name().and_then(|s| s.to_str()) {
                    Some(s) => s.to_string(),
                    None => continue,
                };
                if apps.contains_key(&desktop_id) {
                    continue;
                }
                if let Ok(content) = fs::read_to_string(&path) {
                    let mut in_entry = false;
                    let mut name = None;
                    let mut categories = Vec::new();
                    let mut mimes = Vec::new();
                    let mut nodisplay = false;
                    let mut has_exec = false;

                    for line in content.lines() {
                        let trimmed = line.trim();
                        if trimmed == "[Desktop Entry]" {
                            in_entry = true;
                        } else if trimmed.starts_with('[') && in_entry {
                            break;
                        } else if in_entry {
                            if let Some(val) = trimmed.strip_prefix("Name=") {
                                if name.is_none() {
                                    name = Some(val.trim().to_string());
                                }
                            } else if let Some(val) = trimmed.strip_prefix("Categories=") {
                                categories = val
                                    .split(';')
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty())
                                    .collect();
                            } else if let Some(val) = trimmed.strip_prefix("MimeType=") {
                                mimes = val
                                    .split(';')
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty())
                                    .collect();
                            } else if trimmed == "NoDisplay=true" || trimmed == "Hidden=true" {
                                nodisplay = true;
                            } else if trimmed.starts_with("Exec=") {
                                has_exec = true;
                            }
                        }
                    }

                    if !nodisplay && has_exec {
                        let final_name = name.unwrap_or_else(|| {
                            desktop_id.strip_suffix(".desktop").unwrap_or(&desktop_id).to_string()
                        });
                        apps.insert(desktop_id, (final_name, categories, mimes));
                    }
                }
            }
        }
    }

    let mut browsers = Vec::new();
    let mut file_managers = Vec::new();
    let mut terminals = Vec::new();
    let mut editors = Vec::new();

    for (id, (name, cats, mimes)) in apps {
        let is_browser = cats.iter().any(|c| c.eq_ignore_ascii_case("WebBrowser"))
            || mimes.iter().any(|m| m == "x-scheme-handler/https" || m == "x-scheme-handler/http")
            || id.to_lowercase().contains("firefox")
            || id.to_lowercase().contains("chrome")
            || id.to_lowercase().contains("browser")
            || id.to_lowercase().contains("brave")
            || id.to_lowercase().contains("zen");
        if is_browser {
            browsers.push(AvailableApp {
                desktop_id: id.clone(),
                name: name.clone(),
            });
        }

        let is_fm = cats.iter().any(|c| c.eq_ignore_ascii_case("FileManager") || c.eq_ignore_ascii_case("FileTools"))
            || mimes.iter().any(|m| m == "inode/directory")
            || id.to_lowercase().contains("thunar")
            || id.to_lowercase().contains("nautilus")
            || id.to_lowercase().contains("dolphin")
            || id.to_lowercase().contains("flux")
            || id.to_lowercase().contains("files");
        if is_fm {
            file_managers.push(AvailableApp {
                desktop_id: id.clone(),
                name: name.clone(),
            });
        }

        let is_term = cats.iter().any(|c| c.eq_ignore_ascii_case("TerminalEmulator"))
            || id.to_lowercase().contains("term")
            || id.to_lowercase().contains("alacritty")
            || id.to_lowercase().contains("foot")
            || id.to_lowercase().contains("kitty")
            || id.to_lowercase().contains("ghostty")
            || id.to_lowercase().contains("ptyxis");
        if is_term {
            terminals.push(AvailableApp {
                desktop_id: id.clone(),
                name: name.clone(),
            });
        }

        let is_editor = cats.iter().any(|c| c.eq_ignore_ascii_case("TextEditor") || c.eq_ignore_ascii_case("Development") || c.eq_ignore_ascii_case("IDE"))
            || mimes.iter().any(|m| m == "text/plain")
            || id.to_lowercase().contains("edit")
            || id.to_lowercase().contains("zed")
            || id.to_lowercase().contains("code")
            || id.to_lowercase().contains("nvim")
            || id.to_lowercase().contains("kate");
        if is_editor {
            editors.push(AvailableApp {
                desktop_id: id.clone(),
                name: name.clone(),
            });
        }
    }

    browsers.sort_by(|a, b| a.name.cmp(&b.name));
    file_managers.sort_by(|a, b| a.name.cmp(&b.name));
    terminals.sort_by(|a, b| a.name.cmp(&b.name));
    editors.sort_by(|a, b| a.name.cmp(&b.name));

    (browsers, file_managers, terminals, editors)
}

fn get_current_browser() -> String {
    if let Ok(out) = Command::new("xdg-settings").args(["get", "default-web-browser"]).output() {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !s.is_empty() {
                return s;
            }
        }
    }
    if let Ok(out) = Command::new("xdg-mime").args(["query", "default", "x-scheme-handler/http"]).output() {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !s.is_empty() {
                return s;
            }
        }
    }
    String::new()
}

fn get_current_fm() -> String {
    if let Ok(out) = Command::new("xdg-mime").args(["query", "default", "inode/directory"]).output() {
        if out.status.success() {
            return String::from_utf8_lossy(&out.stdout).trim().to_string();
        }
    }
    String::new()
}

fn get_current_term() -> String {
    if let Ok(out) = Command::new("xdg-mime").args(["query", "default", "x-scheme-handler/terminal"]).output() {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !s.is_empty() {
                return s;
            }
        }
    }
    std::env::var("TERMINAL").unwrap_or_default()
}

fn get_current_editor() -> String {
    if let Ok(out) = Command::new("xdg-mime").args(["query", "default", "text/plain"]).output() {
        if out.status.success() {
            return String::from_utf8_lossy(&out.stdout).trim().to_string();
        }
    }
    String::new()
}

pub fn save_defaults() {
    let state = STATE.lock().unwrap();

    let browser = state.browsers.get(state.current_browser_idx).cloned();
    let fm = state.file_managers.get(state.current_fm_idx).cloned();
    let term = state.terminals.get(state.current_term_idx).cloned();
    let editor = state.editors.get(state.current_editor_idx).cloned();

    drop(state);

    if let Some(b) = browser {
        let _ = Command::new("xdg-settings").args(["set", "default-web-browser", &b.desktop_id]).status();
        let _ = Command::new("xdg-mime").args(["default", &b.desktop_id, "x-scheme-handler/http"]).status();
        let _ = Command::new("xdg-mime").args(["default", &b.desktop_id, "x-scheme-handler/https"]).status();
        let _ = Command::new("xdg-mime").args(["default", &b.desktop_id, "text/html"]).status();
    }

    if let Some(f) = fm {
        let _ = Command::new("xdg-mime").args(["default", &f.desktop_id, "inode/directory"]).status();
    }

    if let Some(t) = term {
        let _ = Command::new("xdg-mime").args(["default", &t.desktop_id, "x-scheme-handler/terminal"]).status();
    }

    if let Some(e) = editor {
        let _ = Command::new("xdg-mime").args(["default", &e.desktop_id, "text/plain"]).status();
    }

    if let Ok(mut state) = STATE.lock() {
        state.status_message = "Preferred default applications saved and applied successfully!".to_string();
    }
}

pub fn cycle_browser(delta: isize) {
    let mut state = STATE.lock().unwrap();
    if state.browsers.is_empty() {
        return;
    }
    let count = state.browsers.len() as isize;
    let next = (state.current_browser_idx as isize + delta).rem_euclid(count);
    state.current_browser_idx = next as usize;
}

pub fn cycle_fm(delta: isize) {
    let mut state = STATE.lock().unwrap();
    if state.file_managers.is_empty() {
        return;
    }
    let count = state.file_managers.len() as isize;
    let next = (state.current_fm_idx as isize + delta).rem_euclid(count);
    state.current_fm_idx = next as usize;
}

pub fn cycle_term(delta: isize) {
    let mut state = STATE.lock().unwrap();
    if state.terminals.is_empty() {
        return;
    }
    let count = state.terminals.len() as isize;
    let next = (state.current_term_idx as isize + delta).rem_euclid(count);
    state.current_term_idx = next as usize;
}

pub fn cycle_editor(delta: isize) {
    let mut state = STATE.lock().unwrap();
    if state.editors.is_empty() {
        return;
    }
    let count = state.editors.len() as isize;
    let next = (state.current_editor_idx as isize + delta).rem_euclid(count);
    state.current_editor_idx = next as usize;
}

pub fn build(page: &mut Page) {
    let state = STATE.lock().unwrap().clone();
    let theme = page.theme;
    let width = page.width;

    // Header info row
    let header_widget = build_header(theme, width, &state.status_message);
    page.row(52.0, header_widget);
    page.end_group();

    // Application Selection Group
    // 1. Web Browser
    let cur_browser_name = state
        .browsers
        .get(state.current_browser_idx)
        .map(|b| b.name.as_str())
        .unwrap_or("None available");
    let browser_ctrl = build_app_selector(theme, cur_browser_name, "browser", cycle_browser);
    let browser_row = row::view(
        theme,
        width,
        row::HEIGHT,
        "Web browser",
        "Default handler for HTTP, HTTPS, and web links",
        browser_ctrl,
        280.0,
    );
    page.row(row::HEIGHT, browser_row);

    // 2. File Manager
    let cur_fm_name = state
        .file_managers
        .get(state.current_fm_idx)
        .map(|f| f.name.as_str())
        .unwrap_or("None available");
    let fm_ctrl = build_app_selector(theme, cur_fm_name, "fm", cycle_fm);
    let fm_row = row::view(
        theme,
        width,
        row::HEIGHT,
        "File manager",
        "Default directory and folder browser (inode/directory)",
        fm_ctrl,
        280.0,
    );
    page.row(row::HEIGHT, fm_row);

    // 3. Terminal Emulator
    let cur_term_name = state
        .terminals
        .get(state.current_term_idx)
        .map(|t| t.name.as_str())
        .unwrap_or("None available");
    let term_ctrl = build_app_selector(theme, cur_term_name, "term", cycle_term);
    let term_row = row::view(
        theme,
        width,
        row::HEIGHT,
        "Terminal emulator",
        "Default shell console and terminal application",
        term_ctrl,
        280.0,
    );
    page.row(row::HEIGHT, term_row);

    // 4. Text Editor
    let cur_editor_name = state
        .editors
        .get(state.current_editor_idx)
        .map(|e| e.name.as_str())
        .unwrap_or("None available");
    let editor_ctrl = build_app_selector(theme, cur_editor_name, "editor", cycle_editor);
    let editor_row = row::view(
        theme,
        width,
        row::HEIGHT,
        "Text editor",
        "Default text file editor (text/plain)",
        editor_ctrl,
        280.0,
    );
    page.row(row::HEIGHT, editor_row);

    page.end_group();

    // Action button group
    let save_btn = button::view(theme, "Save App Defaults", Style::Primary, true, || {
        save_defaults();
    });
    let action_row = row::view(
        theme,
        width,
        row::HEIGHT,
        "Apply application preferences",
        "Write preferred defaults to ~/.config/mimeapps.list and xdg-settings",
        save_btn,
        button::width("Save App Defaults"),
    );
    page.row(row::HEIGHT, action_row);
    page.end_group();
}

fn build_header(theme: &Theme, width: f32, status_msg: &str) -> Rectangle {
    let icon = Text::new("◫").size(16.0).font(fonts::NERD).tight().color(theme.accent);
    let title = Text::new("Preferred Applications (XDG)")
        .size(15.0)
        .font(fonts::BODY)
        .weight(Weight::SemiBold)
        .color(theme.text);
    let subtitle = Text::new(status_msg)
        .size(12.0)
        .font(fonts::BODY)
        .color(theme.muted_text)
        .elide();

    let text_col = Column::new(children![
        Row::new(children![icon, title]).gap(8.0).align(Center),
        subtitle
    ])
    .gap(2.0);

    Rectangle::new()
        .width(width)
        .height(52.0)
        .padding(Padding {
            top: 6.0,
            bottom: 6.0,
            left: 18.0,
            right: 18.0,
        })
        .align_child(Start, Center)
        .child(text_col)
}

fn build_app_selector(
    theme: &Theme,
    app_name: &str,
    _key: &'static str,
    on_cycle: fn(isize),
) -> Rectangle {
    let prev_btn = Rectangle::new()
        .width(28.0)
        .height(28.0)
        .radius(8.0)
        .fill(high_surface(theme))
        .cursor(Pointer)
        .on_click(move |_| on_cycle(-1))
        .align_child(Center, Center)
        .child(Text::new("󰅁").size(14.0).font(fonts::NERD).tight().color(theme.text));

    let next_btn = Rectangle::new()
        .width(28.0)
        .height(28.0)
        .radius(8.0)
        .fill(high_surface(theme))
        .cursor(Pointer)
        .on_click(move |_| on_cycle(1))
        .align_child(Center, Center)
        .child(Text::new("󰅂").size(14.0).font(fonts::NERD).tight().color(theme.text));

    let label_w = 280.0 - 56.0 - 16.0;
    let label = Text::new(app_name)
        .size(13.0)
        .font(fonts::BODY)
        .weight(Weight::SemiBold)
        .color(theme.accent)
        .elide();

    let center_box = Rectangle::new()
        .width(label_w)
        .height(28.0)
        .radius(8.0)
        .fill(high_surface(theme))
        .padding(Padding {
            top: 0.0,
            bottom: 0.0,
            left: 10.0,
            right: 10.0,
        })
        .align_child(Center, Center)
        .child(label);

    let row = Row::new(children![prev_btn, center_box, next_btn])
        .gap(8.0)
        .align(Center);

    Rectangle::new()
        .width(280.0)
        .height(row::CONTROL_HEIGHT)
        .align_child(Start, Center)
        .child(row)
}
