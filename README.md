# READ THIS BEFORE OPENING ANY ISSUE OR PR

this is not a general-purpose shell. it's my own personal shell, and i'm not accepting any PRs or issues for new features. If you wanna add something, fork it yourself.

<div align="center">
  <h1>suzuha</h1>
  <p><strong>My amane config.</strong><br>
  An opinionated Wayland desktop shell written in Rust with amane, built for Niri.</p>

  <p>
    <img alt="amane 0.1" src="https://img.shields.io/badge/AMANE-0.1-89b4fa?style=flat-square&labelColor=181825">
    <img alt="Niri" src="https://img.shields.io/badge/COMPOSITOR-NIRI-89b4fa?style=flat-square&labelColor=181825">
    <img alt="Rust" src="https://img.shields.io/badge/LANGUAGE-RUST-89b4fa?style=flat-square&labelColor=181825">
  </p>
</div>

## Fork Enhancements & Plugins (`custom` branch)

This fork ([`mystrorc/suzuha`](https://github.com/mystrorc/suzuha)) tracks upstream [`MystiaFin/suzuha`](https://github.com/MystiaFin/suzuha) while adding several major features, Omarchy plugin ports, appearance controls, and hardware integrations:

- **Quick Appearance Controls (Utility Panel):**
  - Dedicated **Appearance (`󰏘`)** tab in the Status Bar's Utility overlay (`amane ipc call utility show appearance`).
  - **Colors Subtab:** Mode picker (Auto / Light / Dark), Wallpaper accent toggle, 7 curated accent swatches (Blue, Rose, Peach, Green, Teal, Purple, Pink), and 34 full color schemes.
  - **Cursor Subtab:** 4 cursor size presets (24px, 32px, 48px, 64px), Shake to Find Cursor toggle, and a picker for 10 popular cursor themes.
  - **Fonts Subtab:** Live typography switcher supporting 10 shell font family choices.
- **Shake to Find Cursor Service:**
  - Background cursor velocity and directional reversal detector providing cursor locating feedback.
- **Dynamic Theming & Desktop Integrations:**
  - **Alacritty Terminal:** Native configuration generation writing to `~/.config/alacritty/settings-manager-colors.toml` with instant inotify live reloading.
  - **GTK 3 & GTK 4:** Dynamic CSS symlinks (`~/.config/gtk-3.0/gtk.css` and `~/.config/gtk-4.0/gtk.css`), color-scheme preference, and matching GNOME interface accent-color.
  - **Zed Editor:** Custom theme generation at `~/.config/zed/themes/amane.json` with dynamic syntax highlighting and UI token mapping.
  - **COSMIC & Iced Applications:**
    - Generates COSMIC theme builder and palette files in `~/.config/cosmic/` (supporting both v1 and v2 schemas).
    - Features an embedded, native D-Bus `com.system76.CosmicSettingsDaemon` service bridge inside the Amane shell that provides `WatchConfig`/`WatchState` and broadcasts `Changed` signals, allowing running Iced/COSMIC apps (`cosmic-files`, `cosmic-edit`, etc.) to reload dynamically on the fly without restarting.
- **Screen Recorder Integration:**
  - Native screen recording tab in the Utility Panel with desktop audio and microphone toggles.
  - Powered by `wf-recorder` for lightweight Wayland desktop recording (~5-12% CPU / 30-50MB RAM).
- **Native Monitor Manager & Safety Guard (`src/monitors.rs`):**
  - Integrated Rust background service (`amane::Service`) that monitors Wayland/Niri output states via `event-stream` with 0 extra processes and ~0% CPU.
  - **Undocked Safety Guard:** Automatically detects when the laptop is disconnected from a dock or boots without an external display, guaranteeing `eDP-1` is powered ON at scale `2.0` (`3200x1800@59.982`), preventing black screen lockouts.
  - **Docked Consistency Guard:** Matches external displays by their hardware EDID identity (Make/Model/Serial) rather than transient connector names (`DP-5` vs `DP-1`), preventing USB-C/Thunderbolt dock re-indexing from resetting your scale or layout.
  - **Clamshell Mode Support:** Integrated with Niri's `switch-events` to cleanly power off the laptop display when the lid is closed while docked, and automatically restore it when opened or undocked.
  - **IPC Integration:** `amane ipc call monitors sync` for instant layout synchronization.
- **Monitor Layout Display Editor (`plugins/monitor-layout/`):**
  - Full graphical drag-and-drop display arrangement canvas with edge-snapping, resolution, refresh rate, and scaling picker.
  - Upgraded to persist rules using persistent hardware make/model strings and protected by a safety lock preventing permanent `off` directives for the laptop display.
  - Automatically notifies and synchronizes with Amane's Rust monitor manager upon saving.
- **Power Profile Management:**
  - Interactive profile switcher (Power Saver, Balanced, Performance) integrated directly into the laptop battery card.
  - Controls system power states in real time through `tuned` or `power-profiles-daemon`.
- **Omarchy Plugin Overlays & Tools:**
  - **System Monitor (`src/overlay/sysmon/`):** Real-time monitoring overlay for CPU, memory, and process statistics (`amane ipc call sysmon`).
  - **Audio & 10-Band Parametric Equalizer (`src/overlay/audio_control/`):** Dedicated PipeWire audio control panel with a 10-band graphical equalizer (`amane ipc call audio_control`).
  - **Power Devices Monitor (`src/overlay/power_devices/`):** Per-device battery statistics for laptop battery and connected wireless peripherals (`amane ipc call power`).
- **Settings Manager Plugins Page:**
  - Direct UI launcher for plugins and quick shortcut to open the `~/.config/amane/plugins` folder.

## Preview

https://github.com/user-attachments/assets/019e85b0-a3cb-4189-8a25-d7fd998534ea

## Overview

suzuha is my personal amane configuration: a full desktop shell in Rust rather than a pile of separate widgets. It started as a rewrite of my [JAQC-shell](https://github.com/MystiaFin/shell) Quickshell config, so the two look and feel pretty much the same.

### Included

- Status bar with workspaces, system information, clock, media, and a pomodoro pill
- Liquid shader that lets the bar and panels flow into each other, with rounded screen corners
- Application launcher with a `>` command mode and a tmux project picker
- Control center with media controls and a cava visualizer
- Utility panel with Wi-Fi, Bluetooth, brightness, calendar, notifications, screen recorder, and appearance
- Wallpaper picker with animated transitions and shuffle
- Wallpaper-derived dynamic color palette, plus fixed Gruvbox and Catppuccin schemes
- Floating desktop widgets that place themselves around the wallpaper
- Pomodoro timer
- File converter for images, video, audio, and documents
- Power menu
- logind-driven Wayland lock screen
- Built-in settings window
- Optional theme integrations for GTK, terminals, Alacritty, Zed, COSMIC, tmux, Vesktop, Spotify, btop, and cava

## Installation

Clone the repository as your amane config:

```sh
git clone https://github.com/mystrorc/suzuha.git ~/.config/amane
```

Build the shell, then launch it:

```sh
amane compile
amane run
```

`amane run` only starts the compiled shell, so run `amane compile` again after pulling changes. While editing the config, `amane dev` rebuilds and restarts it on every save.

Or start it with Niri:

```kdl
spawn-at-startup "amane" "run"
```

> [!NOTE]
> This config is built around Niri. Some parts, like the power menu and the wallpaper sitting behind the overview, expect Niri and are not expected to work unchanged on other compositors.

## Dependencies

### Required

- **amane**
- **Niri**
- **Poppins**
- **JetBrains Mono Nerd Font**
- **Symbols Nerd Font**
- **Material Design Icons**
- **curl**

### Optional

These are only needed for their corresponding features:

| Package | Used for |
| --- | --- |
| `cava` | Media visualizer |
| `ffmpeg` / `ffprobe` | Image, video, and audio conversion |
| `ImageMagick 7` | Image conversion |
| `LibreOffice` (`soffice`) | Document conversion |
| `pw-play` / PipeWire | Pomodoro sounds |
| `libnotify` (`notify-send`) | Pomodoro notifications |
| `kitty` remote control | Live kitty palette updates |
| `dconf` / `gsettings` | GTK theme and GNOME accent switching |
| `tmux` | Project launcher and generated tmux palette |
| `xdg-desktop-portal` | Picking a profile picture and accent-color broadcast |
| `wf-recorder` | Screen recording (~5-12% CPU / 30-50MB RAM) |
| `pactl` | Recording desktop sound and mic together |

## IPC

Everything is driven through `amane ipc call`, so it is easy to bind in Niri:

```kdl
Mod+Space { spawn "amane" "ipc" "call" "launcher" "toggle"; }
Mod+Shift+W { spawn "amane" "ipc" "call" "wallpaper" "toggle"; }
```

<details>
<summary><strong>All IPC targets</strong></summary>

```sh
amane ipc call launcher toggle     # also show, hide, showTmux
amane ipc call utility toggle      # also show, hide, then a page: notifications, wifi, bluetooth, record, appearance
amane ipc call control toggle      # also show, hide
amane ipc call wallpaper toggle    # also show, hide
amane ipc call monitors sync       # sync and verify active display states & scale
amane ipc call monitor_layout      # launch the graphical Monitor Layout editor
amane ipc call settings
amane ipc call sysmon              # toggle System Monitor overlay
amane ipc call audio_control       # toggle Audio & Equalizer overlay
amane ipc call power               # toggle Power Devices overlay
amane ipc call record              # start or stop a screen recording
amane ipc call converter
amane ipc call lock
```

</details>

## Launcher

The launcher handles normal application search and also acts as a small command palette.

Type `>` to switch into command mode. Commands can expose things such as:

- Settings
- Wallpapers
- Tmux sessions
- Other shell actions

The available command entries can be enabled or disabled from **Settings → Launcher**.

`showTmux` opens the launcher straight into the tmux project picker:

```kdl
Mod+Shift+P { spawn "amane" "ipc" "call" "launcher" "showTmux"; }
```

## Wallpapers & colors

By default the wallpaper picker reads images from:

```text
~/Pictures/Wallpapers
```

The folder can be changed from **Settings → Wallpaper**.

The selected wallpaper is persisted in:

```text
~/.local/state/amane/wallpaper-selection
```

The wallpaper can also drive the shell's dynamic palette. Wallpaper transitions, shuffle, light/dark mode, and color schemes are configurable from the settings window or the Utility Panel's Appearance tab.

## Lock screen

suzuha includes a Wayland session lock that listens to logind, so anything that asks logind to lock (`loginctl lock-session`, an idle daemon, closing the lid) brings it up. It reuses the active wallpaper and palette, and slides the password field up once you start typing.

Lock it from the power menu, or through IPC:

```sh
amane ipc call lock
```

The name and profile picture shown on the lock screen can be changed under **Settings → User**; they do not change the system account used for authentication.

> [!CAUTION]
> A Wayland session lock deliberately stays locked if the locker dies. If amane crashes while the session is locked, the compositor will not reveal the desktop; recover from another TTY if necessary.

## Pomodoro & converter

Two small tools that live inside the shell instead of being separate apps:

- **Pomodoro** — a focus timer with a wavy progress ring, also shown as a pill on the bar
- **Converter** — converts and compresses images, video, audio, and documents through `ffmpeg`, ImageMagick, and LibreOffice

```sh
amane ipc call converter
```

## Settings

The shell includes its own settings window, so normal day-to-day preferences don't need code changes.

Current sections include:

- Appearance
- User
- Colors
- Wallpaper
- Bar
- Launcher
- Behavior
- Floating widgets
- Weather
- Integrations
- Plugins
- About

Settings are stored in:

```text
~/.local/state/amane/settings
```

## Theme integrations

External theme integrations are **opt-in**. Enabling one may generate configuration files or update a running application, so the shell does not enable them automatically.

Supported integrations currently include:

- GTK 3 / GTK 4 (with GNOME accent-color)
- Alacritty (`settings-manager-colors.toml`)
- Zed Editor (`themes/amane.json`)
- COSMIC / Iced Applications (with built-in D-Bus daemon for live reloads)
- kitty
- foot
- tmux
- Vesktop
- Spotify / Spicetify
- btop
- cava

<details>
<summary><strong>Generated files and side effects</strong></summary>

Depending on which integrations are enabled, suzuha may write files like:

```text
~/.config/alacritty/settings-manager-colors.toml
~/.config/zed/themes/amane.json
~/.config/cosmic/com.system76.CosmicTheme.Mode/v1/is_dark
~/.config/cosmic/com.system76.CosmicTheme.Dark/v2/accent
~/.config/cosmic/com.system76.CosmicTheme.Light/v2/accent
~/.local/state/amane/terminal-colors-kitty.conf
~/.local/state/amane/terminal-colors-foot.ini
~/.local/state/amane/tmux-colors.conf
~/.config/btop/themes/amane.theme
~/.config/cava/themes/amane
~/.cache/amane/theme/spotify.css
```

GTK integration also dynamically symlinks `~/.config/gtk-3.0/gtk.css` and `~/.config/gtk-4.0/gtk.css`, and updates the active color-scheme and GNOME interface accent-color.

For COSMIC / Iced applications, Amane exports a native `com.system76.CosmicSettingsDaemon` D-Bus bridge that broadcasts theme change events directly to running apps for live on-the-fly reloads.

For tmux, add this to `~/.tmux.conf` so new sessions load the generated palette:

```tmux
source-file -q ~/.local/state/amane/tmux-colors.conf
```

For kitty, include the generated colors and let running windows be recolored live:

```conf
allow_remote_control socket-only
listen_on unix:@amane-kitty
include ~/.local/state/amane/terminal-colors-kitty.conf
```

</details>

## Weather

The floating weather widget's location is set under **Settings → Weather** with a place name, latitude, and longitude.

Using approximate city-center coordinates is enough. Weather and air quality data are fetched directly from Open-Meteo; no IP geolocation service is used.

## Project structure

```text
plugins/            standalone python and web plugins (e.g. monitor-layout)
src/bar/            status bar, workspaces, pills, and the pomodoro ring
src/overlay/        launcher, control center, utility panel, power menu, sysmon, eq
src/floating/       floating desktop widgets and their placement
src/wallpaper/      wallpaper, picker, transitions, and shuffle
src/lock_screen/    lock screen and the logind listener
src/settings/       settings window and its pages
src/theme/          palette generation and color schemes
src/integrations/   external theme/application integrations
src/converter/      file converter
src/motion/         springs, glides, and shared animation helpers
shaders/            the liquid shader
src/main.rs         root configuration and IPC routing
```

## Notes

This is a personal config that I daily-drive and keep changing. Expect opinions, occasional breakage, and features that exist because I wanted them on my own desktop.

If you use it as a base for your own setup, reading and modifying the Rust is very much part of the experience.
