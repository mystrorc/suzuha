use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use amane::Service;

pub struct MonitorManager;

impl Service for MonitorManager {
    fn new() -> Self {
        // Enforce safety immediately on startup
        sync_and_guard();
        Self
    }

    fn listen() {
        run_monitor_loop();
    }
}

pub fn ipc(arguments: &[String]) -> String {
    match arguments.first().map(|s| s.as_str()) {
        Some("sync" | "apply" | "reload") => {
            sync_and_guard();
            String::from("synced")
        }
        _ => {
            sync_and_guard();
            String::from("ok")
        }
    }
}

pub fn sync_and_guard() {
    let output = match Command::new("niri").args(["msg", "--json", "outputs"]).output() {
        Ok(out) if out.status.success() => out,
        _ => return,
    };

    let text = match std::str::from_utf8(&output.stdout) {
        Ok(t) => t,
        Err(_) => return,
    };

    let json: serde_json::Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(_) => return,
    };

    let map = match json.as_object() {
        Some(m) => m,
        None => return,
    };

    let mut internal_name = String::from("eDP-1");
    let mut internal_disabled = false;
    let mut internal_found = false;
    let mut external_outputs: Vec<(String, String, serde_json::Value)> = Vec::new();

    for (name, item) in map {
        let is_internal = name.starts_with("eDP")
            || item.get("make").and_then(|m| m.as_str()).map_or(false, |m| m.contains("Sharp"));

        if is_internal {
            internal_found = true;
            internal_name = name.clone();
            internal_disabled = item.get("logical").map_or(true, |l| l.is_null());
        } else {
            let make = item.get("make").and_then(|m| m.as_str()).unwrap_or("");
            let model = item.get("model").and_then(|m| m.as_str()).unwrap_or("");
            let serial = item.get("serial").and_then(|s| s.as_str()).unwrap_or("");
            let ident = format!("{make} {model} {serial}").trim().to_string();
            external_outputs.push((name.clone(), ident, item.clone()));
        }
    }

    // Safety Guard 1: Undocked (No external monitors connected)
    if external_outputs.is_empty() {
        if internal_found && internal_disabled {
            eprintln!("amane-shell: Undocked guard: turning ON internal display {internal_name}");
            let _ = Command::new("niri").args(["msg", "output", &internal_name, "on"]).output();
            let _ = Command::new("niri").args(["msg", "output", &internal_name, "scale", "2.0"]).output();
            let _ = Command::new("niri").args(["msg", "output", &internal_name, "mode", "3200x1800@59.982"]).output();
        }
        return;
    }

    // Safety Guard 2: Docked (External monitors present)
    // When external monitors are present, user preferences (from settings-manager.kdl)
    // govern scaling, modes, and whether eDP-1 is turned off.
    // We do NOT force scale 2.0 or force eDP-1 on when docked.
}

fn run_monitor_loop() {
    loop {
        let child = Command::new("niri")
            .args(["msg", "--json", "event-stream"])
            .stdout(Stdio::piped())
            .spawn();

        match child {
            Ok(mut process) => {
                if let Some(stdout) = process.stdout.take() {
                    let reader = BufReader::new(stdout);
                    for line in reader.lines() {
                        if let Ok(l) = line {
                            if l.contains("WorkspacesChanged") || l.contains("Outputs") || l.contains("output") {
                                sync_and_guard();
                            }
                        } else {
                            break;
                        }
                    }
                }
                let _ = process.kill();
            }
            Err(_) => {
                thread::sleep(Duration::from_secs(5));
                sync_and_guard();
            }
        }

        thread::sleep(Duration::from_secs(2));
    }
}
