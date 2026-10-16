use std::env;
use std::fs::{self, File};
use std::io::Read;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use amane::Service;
use crate::settings::Settings;

static SHAKING: AtomicBool = AtomicBool::new(false);

pub struct ShakeCursor;

impl Service for ShakeCursor {
    fn new() -> Self {
        Self
    }

    fn listen() {
        thread::spawn(run_listener);
    }
}

fn run_listener() {
    let mut buffer = [0u8; 48];

    // Retry loop in case /dev/input/mice is temporarily unavailable
    loop {
        let mut file = match File::open("/dev/input/mice") {
            Ok(f) => f,
            Err(_) => {
                thread::sleep(Duration::from_secs(5));
                continue;
            }
        };

        let mut reversals: u32 = 0;
        let mut last_sign: i8 = 0;
        let mut last_reversal = Instant::now();
        let mut accumulated_dx: i32 = 0;

        while let Ok(count) = file.read(&mut buffer) {
            if count < 3 {
                continue;
            }

            if !Settings::read().flag("shake_cursor") {
                thread::sleep(Duration::from_millis(500));
                continue;
            }

            let mut shake_detected = false;

            // PS/2 mouse packet is 3 bytes: [flags, dx, dy]
            for chunk in buffer[..count].chunks_exact(3) {
                let dx = chunk[1] as i8 as i32;
                if dx.abs() > 10 {
                    let sign = if dx > 0 { 1 } else { -1 };
                    let now = Instant::now();

                    if sign != last_sign {
                        if now.duration_since(last_reversal) < Duration::from_millis(350) {
                            reversals += 1;
                            accumulated_dx += dx.abs();
                        } else {
                            reversals = 1;
                            accumulated_dx = dx.abs();
                        }

                        last_sign = sign;
                        last_reversal = now;

                        if reversals >= 4 && accumulated_dx >= 80 {
                            shake_detected = true;
                            reversals = 0;
                            accumulated_dx = 0;
                            break;
                        }
                    }
                }
            }

            if shake_detected && !SHAKING.load(Ordering::SeqCst) {
                SHAKING.store(true, Ordering::SeqCst);
                thread::spawn(trigger_shake_effect);
            }
        }

        thread::sleep(Duration::from_secs(1));
    }
}

fn trigger_shake_effect() {
    let settings = Settings::read();
    let theme = settings.text("cursor_theme").to_string();
    let normal_size = settings.text("cursor_size").to_string();
    drop(settings);

    let home = env::var("HOME").unwrap_or_default();
    let cursor_kdl = format!("{home}/.config/niri/cursor.kdl");

    // Temporarily enlarge cursor to 64px
    let enlarged_config = format!(
        "cursor {{\n    xcursor-theme \"{theme}\"\n    xcursor-size 64\n}}\n"
    );
    let _ = fs::write(&cursor_kdl, enlarged_config);
    let _ = Command::new("niri").arg("msg").arg("action").arg("load-config-file").spawn();

    // Trigger locate pointer animation if enabled
    let _ = Command::new("gsettings")
        .args(["set", "org.gnome.desktop.interface", "locate-pointer", "true"])
        .spawn();

    thread::sleep(Duration::from_millis(1200));

    // Restore normal size
    let normal_config = format!(
        "cursor {{\n    xcursor-theme \"{theme}\"\n    xcursor-size {normal_size}\n}}\n"
    );
    let _ = fs::write(&cursor_kdl, normal_config);
    let _ = Command::new("niri").arg("msg").arg("action").arg("load-config-file").spawn();

    SHAKING.store(false, Ordering::SeqCst);
}
