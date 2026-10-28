pub mod appearance;
pub mod audio;
mod bar;
pub mod colors;
pub mod default_apps;
mod floating;
mod input;
mod integrations;
pub mod kanata;
mod launcher;
pub mod monitors;
mod weather;

use super::Page as Shown;
use super::page::Page;

// fills the page with the shown page's groups
pub fn build(shown: Shown, page: &mut Page) {
    match shown {
        Shown::Monitors => monitors::build(page),
        Shown::Appearance => appearance::build(page),
        Shown::Input => input::build(page),
        Shown::Audio => audio::build(page),
        Shown::DefaultApps => default_apps::build(page),
        Shown::Launcher => launcher::build(page),
        Shown::Bar => bar::build(page),
        Shown::Floating => floating::build(page),
        Shown::Weather => weather::build(page),
        Shown::Integrations => integrations::build(page),
    }
}
