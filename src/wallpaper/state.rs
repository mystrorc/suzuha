use std::env;
use std::fs;
use std::hash::{BuildHasher, RandomState};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use amane::{Image, Palette, Service};

use crate::motion::{self, Glide};
use crate::settings::Settings;

// the wallpaper picker writes the chosen file here
const SELECTION: &str = ".local/state/amane/wallpaper-selection";

const PALETTE_SIZE: usize = 16;

// the startup animations take this long, a new wallpaper takes what the settings say
pub const DURATION: u64 = 1200;

// a file that never decodes is shown anyway after this long, as nothing
const LOAD_TIMEOUT: Duration = Duration::from_secs(3);

const LOAD_POLL: Duration = Duration::from_millis(16);

/*
 * set by the view once it draws the decoded wallpaper; the window takes a
 * moment to reach the screen at startup, and the intro would play unseen
 */
pub static DRAWN: AtomicBool = AtomicBool::new(false);

/*
 * true while a wallpaper transition is preparing or playing, so background
 * app theme exports wait until the animation finishes instead of spiking CPU
 */
pub static TRANSITIONING: AtomicBool = AtomicBool::new(false);

pub fn is_transitioning() -> bool {
    TRANSITIONING.load(Ordering::Relaxed)
}

/*
 * the chosen wallpaper and the one on screen: a new choice waits until
 * its file is decoded, then grows over the old one as a circle
 */
pub struct Wallpaper {
    // the chosen file, which the palette is made from
    pub path: String,

    // the file on screen, and the one growing over it
    pub shown: String,
    pub incoming: Option<String>,

    // where the circle grows from, as a share of the screen's width and height
    pub center: (f32, f32),

    // how far the circle has grown, 0 to 1
    pub reveal: Glide,

    // at startup the backdrop fades in, then the wallpaper rises from below, both 0 to 1
    pub backdrop: Glide,
    pub rise: Glide,

    // true once the rise has finished, for views that only wait for its end
    pub risen: bool,
}

impl Service for Wallpaper {
    fn new() -> Self {
        let path = read_selection();

        let mut palette = Palette::default();
        palette.open(&path, PALETTE_SIZE);
        *Palette::write() = palette;

        Self {
            shown: path.clone(),
            path,
            incoming: None,
            center: (0.5, 0.5),
            reveal: motion::spatial(0.0, DURATION),
            backdrop: motion::effects(0.0, DURATION),
            rise: motion::spatial(0.0, DURATION),
            risen: false,
        }
    }

    fn listen() {
        // made first, so a choice made during the intro is still seen
        let changes = amane::watch_file(&selection_file());

        play_intro();

        for _ in changes {
            let path = read_selection();

            if path == Self::read().path {
                continue;
            }

            TRANSITIONING.store(true, Ordering::Relaxed);

            Self::write().path = path.clone();

            // Wait until the background image worker has finished decoding the texture
            // while the wallpaper picker smoothly fades out
            wait_until_loaded(&path);

            // Extract the new palette off-lock so the UI render thread never stalls
            if Settings::read().flag("follow_wallpaper") {
                let mut next_palette = Palette::default();
                next_palette.open(&path, PALETTE_SIZE);
                *Palette::write() = next_palette;
            }

            reveal(path);

            TRANSITIONING.store(false, Ordering::Relaxed);
        }
    }
}

fn play_intro() {
    let started = Instant::now();

    while !DRAWN.load(Ordering::Relaxed) && started.elapsed() < LOAD_TIMEOUT {
        thread::sleep(LOAD_POLL);
    }

    Wallpaper::write().backdrop.to(1.0);

    thread::sleep(motion::paced(DURATION));

    Wallpaper::write().rise.to(1.0);

    thread::sleep(motion::paced(DURATION));

    Wallpaper::write().risen = true;
}

// the new wallpaper grows or fades in over the old one, which is replaced once it is covered
fn reveal(path: String) {
    let settings = Settings::read();

    let instant = settings.text("wallpaper_transition") == "instant";
    let duration = settings.number("wallpaper_duration") as u64;

    drop(settings);

    if instant {
        Wallpaper::write().shown = path;

        return;
    }

    let center = (random_share(), random_share());

    // Stage incoming at a tiny non-zero reveal value for one frame so the GPU uploads
    // the texture and mipmaps before the timed reveal glide starts ticking
    {
        let mut wallpaper = Wallpaper::write();

        wallpaper.incoming = Some(path.clone());
        wallpaper.center = center;
        wallpaper.reveal = motion::spatial(0.001, duration);
    }

    thread::sleep(Duration::from_millis(24));

    {
        let mut wallpaper = Wallpaper::write();

        wallpaper.reveal = motion::spatial(0.001, duration);
        wallpaper.reveal.to(1.0);
    }

    thread::sleep(motion::paced(duration));

    let mut wallpaper = Wallpaper::write();

    wallpaper.shown = path;
    wallpaper.incoming = None;
}

fn wait_until_loaded(path: &str) {
    let started = Instant::now();

    while !Image::loaded(path) && started.elapsed() < LOAD_TIMEOUT {
        thread::sleep(LOAD_POLL);
    }
}

// keeps the reveal origin within the middle 35%..65% of the screen for a smoother expansion
pub fn random_share() -> f32 {
    let random = RandomState::new().hash_one(0);

    0.35 + (random % 300) as f32 / 1000.0
}

fn selection_file() -> String {
    let home = env::var("HOME").expect("failed to find home: HOME is not set");

    format!("{home}/{SELECTION}")
}

// the file holds a url like file:///home/me/Pictures/wall.png
fn read_selection() -> String {
    let text = fs::read_to_string(selection_file()).unwrap_or_default();

    let url = text.trim();

    let path = url.strip_prefix("file://").unwrap_or(url);

    String::from(path)
}

// the service sees the file change and reveals the new wallpaper
pub fn choose(path: &str) {
    let url = format!("file://{path}");

    fs::write(selection_file(), url).expect("failed to write wallpaper selection");
}
