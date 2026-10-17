mod bar;
mod clock;
mod converter;
mod floating;
mod fonts;
mod integrations;
mod liquid;
mod lock_screen;
mod monitors;
mod motion;
mod overlay;
mod pomodoro;
mod recorder;
mod profile;
mod screen_mask;
mod settings;
mod shake_cursor;
mod theme;
mod wallpaper;

use amane::{App, Apps, Notifications, Service};

use integrations::Export;
use lock_screen::Logind;
use wallpaper::{Shuffle, Wallpaper};

fn ipc_monitor_layout(_arguments: &[String]) -> String {
    let _ = std::process::Command::new("python3")
        .arg("/home/rcrout/.config/amane/plugins/monitor-layout/monitor_layout.py")
        .spawn();
    String::from("ok")
}

fn main() {
    // reading it once starts the palette before the first frame
    drop(Wallpaper::read());

    // counts down to the next random wallpaper while shuffle is on
    drop(Shuffle::read());

    // the app list is read once up front, so the launcher opens without waiting for it
    drop(Apps::read());

    // the notification server starts here, so nothing sent before the panel first opens is lost
    drop(Notifications::read());

    // listens for logind asking to lock, like `loginctl lock-session` from the power menu
    drop(Logind::read());

    // writes the theme out to the programs turned on under integrations
    drop(Export::read());

    // starts COSMIC settings daemon bridge for iced/cosmic apps live reloading
    integrations::cosmic::start_daemon();

    // pre-initialize background services for system monitor and audio control
    drop(overlay::sysmon::SysmonData::read());
    drop(overlay::audio_control::AudioDevices::read());

    // shake-to-find cursor background service
    drop(shake_cursor::ShakeCursor::read());

    // monitor manager and safety guard
    drop(monitors::MonitorManager::read());

    App::new()
        .font(fonts::body())
        .window_per_monitor(wallpaper::view)
        // over the wallpaper, under everything else
        .window_per_monitor(floating::view)
        // made before the bar and panels, so it sits under them in the same layer
        .window_per_monitor(screen_mask::view)
        .window_per_monitor(bar::view)
        // under the panels, catching clicks outside them when that setting is on
        .window_per_monitor(overlay::dismiss::view)
        .window_per_monitor(overlay::view)
        .window_per_monitor(wallpaper::picker::view)
        .lock(lock_screen::view)
        .ipc("launcher", overlay::launcher::ipc)
        .ipc("utility", overlay::utility::ipc)
        .ipc("control", overlay::control_center::ipc)
        .ipc("sysmon", overlay::sysmon::ipc)
        .ipc("power", overlay::power_devices::ipc)
        .ipc("audio_control", overlay::audio_control::ipc)
        .ipc("monitor_layout", ipc_monitor_layout)
        .ipc("monitors", monitors::ipc)
        .ipc("settings", settings::ipc)
        .ipc("lock", lock_screen::ipc)
        .ipc("wallpaper", wallpaper::picker::ipc)
        .ipc("converter", converter::ipc)
        .ipc("record", recorder::ipc)
        .run();
}
