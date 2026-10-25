use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

#[derive(Default)]
struct ShortcutCache {
    mtime: Option<SystemTime>,
    // key: normalized binary / app target name, value: list of chords
    map: HashMap<String, Vec<String>>,
}

static CACHE: OnceLock<Mutex<ShortcutCache>> = OnceLock::new();

fn get_cache() -> &'static Mutex<ShortcutCache> {
    CACHE.get_or_init(|| Mutex::new(ShortcutCache::default()))
}

fn config_path() -> PathBuf {
    if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
        let p = PathBuf::from(config_home).join("niri/config.kdl");
        if p.exists() {
            return p;
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let p = PathBuf::from(home).join(".config/niri/config.kdl");
        if p.exists() {
            return p;
        }
    }
    PathBuf::from("/home/rcrout/.config/niri/config.kdl")
}

fn parse_config(content: &str) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("/-") {
            continue;
        }

        if !trimmed.contains("spawn") || !trimmed.contains('{') {
            continue;
        }

        let before_brace = match trimmed.split_once('{') {
            Some((b, _)) => b.trim(),
            None => continue,
        };

        let chord_raw = before_brace
            .split_whitespace()
            .next()
            .unwrap_or("");

        if chord_raw.is_empty() {
            continue;
        }

        // Change "Mod" to "Super" for user-friendly shortcut display
        let chord = chord_raw.replace("Mod", "Super").replace("mod", "Super");

        let title = if let Some(t_start) = trimmed.find("hotkey-overlay-title=\"") {
            let rem = &trimmed[t_start + 22..];
            rem.split('"').next().unwrap_or("").to_lowercase()
        } else {
            String::new()
        };

        let after_spawn = if let Some((_, after)) = trimmed.split_once("spawn ") {
            after
        } else if let Some((_, after)) = trimmed.split_once("spawn-sh ") {
            after
        } else {
            continue;
        };

        let mut targets: Vec<String> = Vec::new();

        let mut in_quotes = false;
        let mut current_quote = String::new();
        let mut quoted_strings: Vec<String> = Vec::new();

        for c in after_spawn.chars() {
            if c == '"' {
                if in_quotes {
                    quoted_strings.push(current_quote.clone());
                    current_quote.clear();
                    in_quotes = false;
                } else {
                    in_quotes = true;
                }
            } else if in_quotes {
                current_quote.push(c);
            }
        }

        if !quoted_strings.is_empty() {
            let first_cmd = &quoted_strings[0];
            let first_token = first_cmd.split_whitespace().next().unwrap_or("");
            let bin = Path::new(first_token)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(first_token)
                .to_lowercase();

            if !bin.is_empty() && bin != "sh" && bin != "bash" {
                targets.push(bin.clone());
                if bin == "brave-origin" || bin == "brave-origin-stable" || bin == "brave" {
                    targets.push("brave-origin".to_string());
                    targets.push("brave-origin-stable".to_string());
                    targets.push("brave".to_string());
                    targets.push("brave origin".to_string());
                    targets.push("brave-browser".to_string());
                }
            }

            if bin == "amane" && quoted_strings.len() >= 4 && quoted_strings[1] == "ipc" && quoted_strings[2] == "call" {
                targets.push(quoted_strings[3].to_lowercase());
            }

            if bin == "sh" || bin == "bash" {
                for token in first_cmd.split_whitespace() {
                    let token_clean = token.trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '_');
                    if token_clean.len() >= 3 && token_clean != "pkill" && token_clean != "exec" && token_clean != "set" {
                        targets.push(token_clean.to_lowercase());
                    }
                }
            }
        }

        if !title.is_empty() {
            for word in title.split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_') {
                let w = word.trim();
                if w.len() >= 3
                    && w != "open"
                    && w != "the"
                    && w != "run"
                    && w != "file"
                    && w != "terminal"
                    && w != "manager"
                    && w != "application"
                    && w != "settings"
                    && w != "picker"
                {
                    targets.push(w.to_lowercase());
                }
            }
        }

        for target in targets {
            let entry = map.entry(target).or_default();
            if !entry.contains(&chord) {
                entry.push(chord.clone());
            }
        }
    }

    map
}

pub fn get_shortcut(app_name: &str, exec: &str) -> Option<String> {
    let path = config_path();
    let current_mtime = fs::metadata(&path).and_then(|m| m.modified()).ok();

    let mut cache = get_cache().lock().unwrap();
    if cache.mtime != current_mtime || cache.mtime.is_none() {
        if let Ok(content) = fs::read_to_string(&path) {
            cache.map = parse_config(&content);
            cache.mtime = current_mtime;
        }
    }

    let tokens: Vec<&str> = exec.split_whitespace().collect();
    let mut candidate_bins: Vec<String> = Vec::new();

    if tokens.len() >= 3 && (tokens[1] == "-e" || tokens[1] == "--") {
        let inner_bin = Path::new(tokens[2])
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(tokens[2])
            .to_lowercase();
        candidate_bins.push(inner_bin);
    } else if !tokens.is_empty() {
        let first_bin = Path::new(tokens[0])
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(tokens[0])
            .to_lowercase();
        candidate_bins.push(first_bin.clone());
        if first_bin == "brave-origin-stable" || first_bin == "brave-origin" || first_bin == "brave" {
            candidate_bins.push("brave-origin".to_string());
            candidate_bins.push("brave-origin-stable".to_string());
            candidate_bins.push("brave".to_string());
        }
    }

    let name_clean = app_name.to_lowercase();
    let name_hyphen = name_clean.replace(' ', "-");

    for bin in &candidate_bins {
        if let Some(chords) = cache.map.get(bin) {
            return Some(chords.join(", "));
        }
    }

    if let Some(chords) = cache.map.get(&name_clean) {
        return Some(chords.join(", "));
    }

    if let Some(chords) = cache.map.get(&name_hyphen) {
        return Some(chords.join(", "));
    }

    None
}

pub fn notify_shortcut_hint(app_name: &str, shortcut: &str, icon: &str) {
    let icon_arg = if icon.is_empty() { "input-keyboard" } else { icon };
    let cmd = format!(
        "notify-send -a 'Key Promoter' -i '{icon_arg}' 'Key Promoter' 'Use shortcut {shortcut} to launch {app_name}' -t 3500"
    );
    amane::spawn(&cmd);
}

pub fn record_unmapped_and_notify_if_needed(app_name: &str, icon: &str) {
    let home = std::env::var("HOME").unwrap_or_else(|_| String::from("/home/rcrout"));
    let state_dir = PathBuf::from(&home).join(".local/state/key-promoter");
    let _ = fs::create_dir_all(&state_dir);
    let state_file = state_dir.join("daily_stats.json");

    let today = amane::output("date +%Y-%m-%d").trim().to_string();

    #[derive(serde::Serialize, serde::Deserialize, Default)]
    struct State {
        date: String,
        counts: HashMap<String, u32>,
        notified: Vec<String>,
    }

    let mut state: State = fs::read_to_string(&state_file)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    if state.date != today {
        state.date = today;
        state.counts.clear();
        state.notified.clear();
    }

    let count = state.counts.entry(app_name.to_string()).or_insert(0);
    *count += 1;

    if *count >= 5 && !state.notified.contains(&app_name.to_string()) {
        state.notified.push(app_name.to_string());
        let icon_arg = if icon.is_empty() { "input-keyboard" } else { icon };
        let cmd = format!(
            "notify-send -a 'Key Promoter' -i '{icon_arg}' 'Key Promoter' 'You have launched {app_name} 5 times today without a shortcut.\nConsider adding a shortcut in ~/.config/niri/config.kdl' -t 6000"
        );
        amane::spawn(&cmd);
    }

    if let Ok(json) = serde_json::to_string_pretty(&state) {
        let _ = fs::write(&state_file, json);
    }
}
