use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{LazyLock, Mutex};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum KanataState {
    Active,
    Inactive,
    NotConfigured,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemapRule {
    pub trigger: String,
    pub action: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanataProfile {
    pub name: String,
    pub device: String,
    pub rules: Vec<RemapRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KanataDeviceInfo {
    pub id: String,
    pub display_name: String,
    pub path: String,
    pub description: String,
}

impl KanataDeviceInfo {
    pub fn default_all() -> Self {
        Self {
            id: "All Devices (Default)".to_string(),
            display_name: "All Keyboards & Devices (Default)".to_string(),
            path: "All Devices (Default)".to_string(),
            description: "Applies remap rules globally to all connected keyboards and input devices.".to_string(),
        }
    }
}

pub const ACTION_PRESETS: &[(&str, &str, &str)] = &[
    ("Volume Up (+5%)", "volu", "Volume Up (+5%)"),
    ("Volume Down (-5%)", "vold", "Volume Down (-5%)"),
    ("Volume Mute / Unmute", "mute", "Toggle Volume Mute"),
    ("Play / Pause Media", "pp", "Play / Pause Media"),
    ("Next Track", "next", "Next Track"),
    ("Previous Track", "prev", "Previous Track"),
    ("Stop Media", "stop", "Stop Media Playback"),
    ("Brightness Up", "brup", "Increase Screen Brightness"),
    ("Brightness Down", "brdn", "Decrease Screen Brightness"),
    ("Escape (esc)", "esc", "Escape Key"),
    ("Caps Lock (caps)", "caps", "Caps Lock Key"),
    ("Left Control (lctl)", "lctl", "Left Control"),
    ("Left Alt (lalt)", "lalt", "Left Alt"),
    ("Left Super / Windows (lmet)", "lmet", "Left Super / Windows"),
    ("Right Alt / AltGr (ralt)", "ralt", "Right Alt / AltGr"),
    ("Backspace (bspc)", "bspc", "Backspace Key"),
    ("Enter / Return (ret)", "ret", "Enter / Return Key"),
    ("Tab (tab)", "tab", "Tab Key"),
    ("Delete (del)", "del", "Delete Key"),
    ("Space (spc)", "spc", "Spacebar"),
    ("Print Screen (prnt)", "prnt", "Print Screen / Screenshot"),
    ("Scroll Lock (slck)", "slck", "Scroll Lock Key"),
    ("Arrow Up (up)", "up", "Arrow Up"),
    ("Arrow Down (down)", "down", "Arrow Down"),
    ("Arrow Left (left)", "left", "Arrow Left"),
    ("Arrow Right (rght)", "rght", "Arrow Right"),
    ("Page Up (pgup)", "pgup", "Page Up"),
    ("Page Down (pgdn)", "pgdn", "Page Down"),
    ("Home (home)", "home", "Home Key"),
    ("End (end)", "end", "End Key"),
    ("Insert (ins)", "ins", "Insert Key"),
    ("Pause / Break (pause)", "pause", "Pause / Break Key"),
    ("F1", "f1", "F1 Function Key"),
    ("F2", "f2", "F2 Function Key"),
    ("F3", "f3", "F3 Function Key"),
    ("F4", "f4", "F4 Function Key"),
    ("F5", "f5", "F5 Function Key"),
    ("F6", "f6", "F6 Function Key"),
    ("F7", "f7", "F7 Function Key"),
    ("F8", "f8", "F8 Function Key"),
    ("F9", "f9", "F9 Function Key"),
    ("F10", "f10", "F10 Function Key"),
    ("F11", "f11", "F11 Function Key"),
    ("F12", "f12", "F12 Function Key"),
    ("Tap for Escape, Hold for Left Control", "tap-hold 200 200 esc lctl", "Tap for Escape, Hold for Ctrl"),
    ("Tap for Backspace, Hold for Left Alt", "tap-hold 200 200 bspc lalt", "Tap for Backspace, Hold for Alt"),
    ("Tap for Enter, Hold for Left Super", "tap-hold 200 200 ret lmet", "Tap for Enter, Hold for Super"),
    ("Type Text / Macro...", "__TEXT__", "Type Custom Text"),
    ("Custom Kanata Syntax...", "__CUSTOM__", "Custom Kanata Action"),
];

pub const TRIGGER_PRESETS: &[(&str, &str)] = &[
    ("Scroll Lock (slck)", "slck"),
    ("Caps Lock (caps)", "caps"),
    ("Right Alt / AltGr (ralt)", "ralt"),
    ("Insert (ins)", "ins"),
    ("Pause / Break (pause)", "pause"),
    ("Menu / Apps (comp)", "comp"),
    ("Print Screen (prnt)", "prnt"),
    ("Tilde / Backtick ` (grv)", "grv"),
    ("Right Control (rctl)", "rctl"),
    ("Escape (esc)", "esc"),
    ("F1", "f1"),
    ("F2", "f2"),
    ("F3", "f3"),
    ("F4", "f4"),
    ("F5", "f5"),
    ("F6", "f6"),
    ("F7", "f7"),
    ("F8", "f8"),
    ("F9", "f9"),
    ("F10", "f10"),
    ("F11", "f11"),
    ("F12", "f12"),
];

pub fn text_to_kanata_macro(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let mut tokens = Vec::new();
    tokens.push("macro".to_string());
    for ch in text.chars() {
        match ch {
            'a'..='z' => tokens.push(ch.to_string()),
            '0'..='9' => tokens.push(format!("@num{}", ch)),
            'A'..='Z' => tokens.push(format!("S-{}", ch.to_ascii_lowercase())),
            ' ' => tokens.push("spc".to_string()),
            '\n' => tokens.push("ret".to_string()),
            '\t' => tokens.push("tab".to_string()),
            '!' => tokens.push("S-1".to_string()),
            '@' => tokens.push("S-2".to_string()),
            '#' => tokens.push("S-3".to_string()),
            '$' => tokens.push("S-4".to_string()),
            '%' => tokens.push("S-5".to_string()),
            '^' => tokens.push("S-6".to_string()),
            '&' => tokens.push("S-7".to_string()),
            '*' => tokens.push("S-8".to_string()),
            '(' => tokens.push("S-9".to_string()),
            ')' => tokens.push("S-0".to_string()),
            '-' => tokens.push("min".to_string()),
            '_' => tokens.push("S-min".to_string()),
            '=' => tokens.push("eql".to_string()),
            '+' => tokens.push("S-eql".to_string()),
            '[' => tokens.push("[".to_string()),
            '{' => tokens.push("S-[".to_string()),
            ']' => tokens.push("]".to_string()),
            '}' => tokens.push("S-]".to_string()),
            ';' => tokens.push(";".to_string()),
            ':' => tokens.push("S-;".to_string()),
            '\'' => tokens.push("'".to_string()),
            '"' => tokens.push("S-'".to_string()),
            ',' => tokens.push(",".to_string()),
            '<' => tokens.push("S-,".to_string()),
            '.' => tokens.push(".".to_string()),
            '>' => tokens.push("S-.".to_string()),
            '/' => tokens.push("/".to_string()),
            '?' => tokens.push("S-/".to_string()),
            '\\' => tokens.push("\\".to_string()),
            '|' => tokens.push("S-\\".to_string()),
            '`' => tokens.push("grv".to_string()),
            '~' => tokens.push("S-grv".to_string()),
            _ => tokens.push(format!("(unicode U+{:04X})", ch as u32)),
        }
    }
    format!("({})", tokens.join(" "))
}

pub fn fix_macro_digits(act: &str) -> String {
    if !act.contains("macro") {
        return act.to_string();
    }
    let mut result = String::new();
    let mut chars = act.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '(' || ch == ')' || ch.is_whitespace() {
            result.push(ch);
        } else {
            let mut word = String::new();
            word.push(ch);
            while let Some(&next_ch) = chars.peek() {
                if next_ch == '(' || next_ch == ')' || next_ch.is_whitespace() {
                    break;
                }
                word.push(chars.next().unwrap());
            }
            if word.len() == 1 && word.chars().next().unwrap().is_ascii_digit() {
                result.push_str(&format!("@num{}", word));
            } else {
                result.push_str(&word);
            }
        }
    }
    result
}

pub fn scan_input_devices() -> Vec<KanataDeviceInfo> {
    let mut devices = Vec::new();
    devices.push(KanataDeviceInfo::default_all());

    let mut node_to_ref: BTreeMap<String, (String, String)> = BTreeMap::new();

    let p_by_id = Path::new("/dev/input/by-id");
    if let Ok(entries) = fs::read_dir(p_by_id) {
        for entry in entries.flatten() {
            let fname = entry.file_name().to_string_lossy().to_string();
            if fname.starts_with('.')
                || fname.ends_with("-mouse")
                || fname.contains("-mouse")
                || fname.contains("hidraw")
            {
                continue;
            }
            let full = p_by_id.join(&fname);
            if let Ok(real) = fs::canonicalize(&full) {
                if let Some(ev) = real.file_name().and_then(|n| n.to_str()) {
                    if ev.starts_with("event") && !node_to_ref.contains_key(ev) {
                        node_to_ref.insert(ev.to_string(), (full.to_string_lossy().to_string(), fname));
                    }
                }
            }
        }
    }

    let p_by_path = Path::new("/dev/input/by-path");
    if let Ok(entries) = fs::read_dir(p_by_path) {
        for entry in entries.flatten() {
            let fname = entry.file_name().to_string_lossy().to_string();
            if fname.starts_with('.')
                || fname.ends_with("-mouse")
                || fname.contains("-mouse")
                || fname.contains("hidraw")
            {
                continue;
            }
            let full = p_by_path.join(&fname);
            if let Ok(real) = fs::canonicalize(&full) {
                if let Some(ev) = real.file_name().and_then(|n| n.to_str()) {
                    if ev.starts_with("event") && !node_to_ref.contains_key(ev) {
                        node_to_ref.insert(ev.to_string(), (full.to_string_lossy().to_string(), fname));
                    }
                }
            }
        }
    }

    struct Candidate {
        id: String,
        display_name: String,
        path: String,
        description: String,
        order: u8,
        hw_name: String,
        ev: String,
    }

    let mut candidates = Vec::new();

    for (ev, (full_path, ref_name)) in &node_to_ref {
        let sys_dir = PathBuf::from(format!("/sys/class/input/{}", ev));
        let name_path = sys_dir.join("device/name");
        let Ok(hw_name_raw) = fs::read_to_string(&name_path) else {
            continue;
        };
        let hw_name = hw_name_raw.trim().to_string();
        if hw_name.is_empty() {
            continue;
        }

        let lower_hw = hw_name.to_lowercase();
        if lower_hw.starts_with("kanata") || lower_hw.starts_with("input-remapper") {
            continue;
        }

        if lower_hw.contains("touchpad")
            || lower_hw.contains("mouse")
            || lower_hw.contains("finger")
            || lower_hw.contains("pen")
            || lower_hw.contains("lid switch")
            || lower_hw.contains("video bus")
            || lower_hw.contains("power button")
            || lower_hw.contains("sleep button")
            || lower_hw.contains("pc speaker")
        {
            continue;
        }

        let cap_key_path = sys_dir.join("device/capabilities/key");
        if let Ok(k) = fs::read_to_string(&cap_key_path) {
            let trimmed = k.trim();
            if trimmed.is_empty() || trimmed == "0" {
                continue;
            }
        } else {
            continue;
        }

        let (role, order) = if ref_name.ends_with("-kbd") || ref_name.contains("-event-kbd") {
            if ref_name.contains("serio") || ref_name.contains("i8042") || lower_hw.contains("at translated") {
                ("Built-in Keyboard", 1)
            } else {
                ("Primary Keyboard", 1)
            }
        } else if ref_name.contains("serio") || ref_name.contains("i8042") || lower_hw.contains("at translated") {
            ("Built-in Keyboard", 1)
        } else if ref_name.contains("-if01")
            || ref_name.contains("-if02")
            || ref_name.contains("-event-if")
            || lower_hw.contains("consumer control")
            || lower_hw.contains("system control")
        {
            ("Media / Extra Keys", 2)
        } else if lower_hw.contains("soundbar") || lower_hw.contains("audio") {
            ("Audio Keys", 4)
        } else if lower_hw.contains("button")
            || lower_hw.contains("hotkey")
            || lower_hw.contains("switch")
            || lower_hw.contains("hid")
        {
            ("System Hotkeys", 4)
        } else {
            ("Input Device", 3)
        };

        let display = format!("{} ({})", hw_name, role);
        let detail = format!("{} • {} • {} ({})", hw_name, role, full_path, ev);

        candidates.push(Candidate {
            id: ref_name.clone(),
            display_name: display,
            path: full_path.clone(),
            description: detail,
            order,
            hw_name,
            ev: ev.clone(),
        });
    }

    candidates.sort_by(|a, b| {
        a.order
            .cmp(&b.order)
            .then_with(|| a.hw_name.cmp(&b.hw_name))
            .then_with(|| a.ev.cmp(&b.ev))
    });

    let mut name_counts: HashMap<String, usize> = HashMap::new();
    for c in &candidates {
        *name_counts.entry(c.display_name.clone()).or_insert(0) += 1;
    }

    for mut c in candidates {
        if name_counts.get(&c.display_name).copied().unwrap_or(0) > 1 {
            c.display_name = format!("{} [{}]", c.display_name, c.ev);
        }
        devices.push(KanataDeviceInfo {
            id: c.id,
            display_name: c.display_name,
            path: c.path,
            description: c.description,
        });
    }

    devices
}

pub fn find_kanata_bin() -> Option<PathBuf> {
    if let Some(paths) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&paths) {
            let candidate = dir.join("kanata");
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/home/rcrout"));
    let local_bin = home.join(".local/bin/kanata");
    if local_bin.is_file() {
        return Some(local_bin);
    }
    let usr_local = PathBuf::from("/usr/local/bin/kanata");
    if usr_local.is_file() {
        return Some(usr_local);
    }
    let usr_bin = PathBuf::from("/usr/bin/kanata");
    if usr_bin.is_file() {
        return Some(usr_bin);
    }
    None
}

pub fn ensure_systemd_unit() -> std::io::Result<()> {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/home/rcrout"));
    let unit_dir = home.join(".config/systemd/user");
    let _ = fs::create_dir_all(&unit_dir);
    let unit_file = unit_dir.join("kanata.service");

    let bin_path = find_kanata_bin()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "/home/rcrout/.local/bin/kanata".to_string());

    let unit_content = format!(
        "[Unit]\nDescription=Kanata key remapper daemon\nDocumentation=https://github.com/jtroo/kanata\n\n[Service]\nType=simple\nExecStart={} --cfg %h/.config/kanata/kanata.kbd --nodelay\nRestart=always\nRestartSec=3\n\n[Install]\nWantedBy=default.target\n",
        bin_path
    );

    fs::write(&unit_file, unit_content)
}

pub fn config_file_path() -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/home/rcrout"));
    home.join(".config/kanata/kanata.kbd")
}

pub fn default_profile() -> KanataProfile {
    KanataProfile {
        name: "Default Profile".to_string(),
        device: "All Devices (Default)".to_string(),
        rules: vec![
            RemapRule {
                trigger: "caps".to_string(),
                action: "tap-hold 200 200 esc lctl".to_string(),
                description: "Tap for Escape, Hold for Left Control".to_string(),
            },
            RemapRule {
                trigger: "slck".to_string(),
                action: "volu".to_string(),
                description: "Scroll Lock (slck) to Volume Up (+5%)".to_string(),
            },
        ],
    }
}

pub fn load_profile() -> KanataProfile {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/home/rcrout"));
    let profile_file = home.join(".config/kanata/profile.json");
    if let Ok(content) = fs::read_to_string(&profile_file) {
        if let Ok(mut profile) = serde_json::from_str::<KanataProfile>(&content) {
            for r in &mut profile.rules {
                r.action = fix_macro_digits(&r.action);
            }
            return profile;
        }
    }
    default_profile()
}

pub fn save_profile(profile: &KanataProfile) -> std::io::Result<()> {
    let mut fixed_profile = profile.clone();
    for r in &mut fixed_profile.rules {
        r.action = fix_macro_digits(&r.action);
    }

    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/home/rcrout"));
    let dir = home.join(".config/kanata");
    let _ = fs::create_dir_all(&dir);

    let profile_file = dir.join("profile.json");
    let json = serde_json::to_string_pretty(&fixed_profile)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    fs::write(&profile_file, json)?;

    generate_kbd_file(&fixed_profile, &dir.join("kanata.kbd"))?;
    let _ = ensure_systemd_unit();
    Ok(())
}

pub fn generate_kbd_file(profile: &KanataProfile, path: &Path) -> std::io::Result<()> {
    let mut s = String::new();
    s.push_str(";; Generated by Settings Manager / Amane Input Remapper\n");
    s.push_str(";; Profile: ");
    s.push_str(&profile.name);
    s.push_str("\n\n(defcfg\n  process-unmapped-keys yes\n");

    if !profile.device.is_empty()
        && !profile.device.contains("All Devices")
        && !profile.device.contains("All Keyboards")
    {
        let dev_path = if profile.device.starts_with("/dev/") {
            profile.device.clone()
        } else if Path::new(&format!("/dev/input/by-id/{}", profile.device)).exists() {
            format!("/dev/input/by-id/{}", profile.device)
        } else if Path::new(&format!("/dev/input/by-path/{}", profile.device)).exists() {
            format!("/dev/input/by-path/{}", profile.device)
        } else {
            format!("/dev/input/by-id/{}", profile.device)
        };
        s.push_str(&format!("  linux-dev \"{}\"\n", dev_path));
    }
    s.push_str(")\n\n");

    if profile.rules.is_empty() {
        s.push_str("(defsrc)\n(deflayer default)\n");
        return fs::write(path, s);
    }

    s.push_str("(defsrc\n");
    for r in &profile.rules {
        s.push_str(&format!("  {}\n", r.trigger.trim()));
    }
    s.push_str(")\n\n");

    let mut aliases = Vec::new();
    let mut deflayer_actions = Vec::new();

    aliases.push("  num0 0\n  num1 1\n  num2 2\n  num3 3\n  num4 4\n  num5 5\n  num6 6\n  num7 7\n  num8 8\n  num9 9".to_string());

    for (i, r) in profile.rules.iter().enumerate() {
        let fixed_act = fix_macro_digits(&r.action);
        let act = fixed_act.trim();
        let is_complex = act.contains(' ') || act.starts_with('(');
        if is_complex {
            let alias_name = format!("act{}", i);
            let formatted = if act.starts_with('(') && act.ends_with(')') {
                act.to_string()
            } else {
                format!("({})", act)
            };
            aliases.push(format!("  {} {}", alias_name, formatted));
            deflayer_actions.push(format!("  @{}", alias_name));
        } else {
            deflayer_actions.push(format!("  {}", act));
        }
    }

    if !aliases.is_empty() {
        s.push_str("(defalias\n");
        for a in aliases {
            s.push_str(&a);
            s.push('\n');
        }
        s.push_str(")\n\n");
    }

    s.push_str("(deflayer default\n");
    for a in deflayer_actions {
        s.push_str(&a);
        s.push('\n');
    }
    s.push_str(")\n");

    fs::write(path, s)
}

pub fn is_service_active() -> bool {
    if let Ok(out) = Command::new("systemctl")
        .args(["--user", "is-active", "kanata"])
        .output()
    {
        if out.status.success() && String::from_utf8_lossy(&out.stdout).trim() == "active" {
            return true;
        }
    }
    if let Ok(out) = Command::new("pgrep").args(["-x", "kanata"]).output() {
        if out.status.success() {
            return true;
        }
    }
    false
}

pub fn apply_and_restart(profile: &KanataProfile) -> Result<String, String> {
    save_profile(profile).map_err(|e| format!("Failed to save Kanata config: {e}"))?;
    let _ = ensure_systemd_unit();

    let _ = Command::new("systemctl").args(["--user", "daemon-reload"]).status();
    let _ = Command::new("systemctl").args(["--user", "enable", "kanata"]).status();

    if let Ok(status) = Command::new("systemctl").args(["--user", "restart", "kanata"]).status() {
        if status.success() {
            return Ok("Kanata daemon reloaded successfully".to_string());
        }
    }

    if let Some(bin) = find_kanata_bin() {
        let cfg = config_file_path();
        let _ = Command::new("nohup")
            .arg(bin)
            .args(["--cfg", &cfg.to_string_lossy(), "--nodelay"])
            .spawn();
        return Ok("Kanata daemon spawned in background".to_string());
    }

    Err("Kanata binary not found. Please install kanata.".to_string())
}

#[derive(Clone)]
pub struct RemapperState {
    pub devices: Vec<KanataDeviceInfo>,
    pub selected_device_idx: usize,
    pub profile: KanataProfile,
    pub trigger_preset_idx: usize,
    pub action_preset_idx: usize,
    pub custom_text: String,
    pub custom_syntax: String,
    pub rule_desc: String,
    pub status_message: String,
    pub is_active: bool,
}

impl Default for RemapperState {
    fn default() -> Self {
        let profile = load_profile();
        let devices = scan_input_devices();
        let selected_device_idx = devices
            .iter()
            .position(|d| d.id == profile.device || d.path == profile.device)
            .unwrap_or(0);
        let is_active = is_service_active();

        Self {
            devices,
            selected_device_idx,
            profile,
            trigger_preset_idx: 0,
            action_preset_idx: 0,
            custom_text: String::new(),
            custom_syntax: String::new(),
            rule_desc: String::new(),
            status_message: if is_active {
                "Kanata service is running and active".to_string()
            } else {
                "Kanata service is currently inactive".to_string()
            },
            is_active,
        }
    }
}

pub static REMAPPER: LazyLock<Mutex<RemapperState>> = LazyLock::new(|| Mutex::new(RemapperState::default()));
