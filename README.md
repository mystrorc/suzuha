# Swifty Shell

<div align="center">
  <h1>Swifty</h1>
  <p><strong>A modern, opinionated Wayland desktop shell written in Rust with Amane, built for Niri.</strong></p>

  <p>
    <img alt="amane 0.1" src="https://img.shields.io/badge/AMANE-0.1-89b4fa?style=flat-square&labelColor=181825">
    <img alt="Niri" src="https://img.shields.io/badge/COMPOSITOR-NIRI-89b4fa?style=flat-square&labelColor=181825">
    <img alt="Rust" src="https://img.shields.io/badge/LANGUAGE-RUST-89b4fa?style=flat-square&labelColor=181825">
  </p>
</div>

## Features & Highlights

Swifty ([`mystrorc/swifty`](https://github.com/mystrorc/swifty)) is an opinionated, highly refined desktop shell built with Rust and Amane for the Niri Wayland compositor:

- **Dynamic Network Tray Icon & Spaced System Tray (`src/bar/system.rs`):**
  - **Dynamic Link Icon:** Automatically detects the active connection state via NetworkManager/Amane services. Displays an Ethernet LAN icon (`󰈀`) when wired, Wi-Fi (``) when wireless, and disconnected (`󰤭`) when offline.
  - **Balanced Icon Spacing:** Fixed previous glyph crowding between Wi-Fi and Bluetooth with dedicated 18px icon bounding slots and an explicit 10px visual gap inside the system tray pill.
- **Top Bar Settings Pill & Pulldown Panel (`src/overlay/settings_menu.rs`, `src/bar/workspaces.rs`):**
  - **Settings Bar Pill (`󰒓`):** Positioned on the top-left immediately to the right of the distro logo, styled with a 32px height flush capsule.
  - **Animated Liquid Settings Pulldown:** Clicking the pill drops down an interactive preferences panel with category subtabs:
    - **Appearance:** Color Mode (Auto / Light / Dark), Animation Speed (0.5× / 1.0× / 1.5× / 2.0×), Wallpaper Transitions (Circle / Wipe / Wave / Fade), Transparency, and Shake-to-Find cursor toggles.
    - **Bar:** Toggle Distro Logo, System Tray, Power Button pills, 24-Hour time, seconds, and click-outside dismissal.
    - **Launcher:** Toggle command mode, descriptions, application icons, query memory, and close-on-launch.
    - **Input:** Touchpad tap-to-click, natural scrolling, disable-while-typing, flat mouse acceleration, and left-handed mode.
    - **Integrations:** COSMIC, Zed Editor, GTK 3 & 4, and Alacritty terminal color synchronization.
    - **Full Settings Shortcut:** One-click launcher to open the complete multi-window Settings app.
- **Flush Top Bar (`y = 0` Edge Alignment):**
  - Top bar pills and buttons sit flush against the top screen edge with standardized 32px height, ensuring zero dead space at the top of displays.
- **Conditional Home Assistant Smart Home Pill & Flyout (`src/bar/home_assistant.rs`, `src/overlay/home_assistant.rs`):**
  - **Conditional Bar Pill (`󰌵`):** Positioned immediately to the right of the battery pill; dynamically shown only when connected to the home network and Home Assistant is reachable. Displays an illuminated icon and active light count badge when lights are on.
  - **Quick Action:** Right-click directly on the bar pill turns off all lights instantly without opening any menu.
  - **Liquid Popover Flyout:** Left-clicking drops down an animated liquid flyout with live status, optimistic toggle switches for all discovered Home Assistant light entities, and an "All Off" shortcut.
  - **Configuration:** Credentials securely stored in `~/.config/amane/homeassistant.json`.
- **Redesigned Calendar & 7-Day Weather Pulldown (`src/overlay/control_center/calendar.rs`):**
  - Expanded Control Center dropdown (820px) featuring a live 7-day weather forecast powered by Open-Meteo.
  - Large digital clock card with day-of-week and month indicators.
  - Local agenda and persistent reminders list backed by `~/.local/state/amane/reminders.json`.
  - Monthly calendar grid with circular today highlights and active event indicator dots.
- **Unified Media, Audio & Equalizer Tab (`src/overlay/control_center/media.rs`, `src/overlay/control_center/audio_eq.rs`):**
  - Combined Now Playing and Audio & EQ into a unified interface.
  - Left card: Compact volume and microphone faders side-by-side with PipeWire output sink selector and Wiremix integration directly underneath (input selector removed).
  - Right card: Now Playing with album art, live Cava audio visualizer, seek waveform, and playback controls on top, and 10-band parametric equalizer on the bottom.
  - Calibrated AutoEQ profiles for **Sony WH-1000XM3** and Razer Kraken TE, with an expanded `[-15.0 dB, +15.0 dB]` fader range.
- **System Info & Hardware Overview (`src/settings/pages/user.rs`):**
  - Displays a Hero System Card with the distribution logo, OS name, kernel release, dynamic system uptime, and CPU summary.
  - Includes right-aligned hardware specification rows for Operating System, Kernel Release, System Uptime, Installed Packages (`rpm`), Processor, Total Memory, and Graphics Adapter (`lspci`), positioned directly above the lock screen User Profile configuration.
- **Input & Per-Device Pointer Configuration (`src/settings/pages/input.rs`):**
  - Dedicated **Input (`󰌌`)** tab in Amane Settings for keyboard, touchpad, mouse, and trackball configuration.
  - **Keyboard:** Layout, repeat rate (chars/sec), and repeat delay (ms).
  - **Touchpad:** Natural scrolling, tap-to-click, and disable-while-typing toggles.
  - **Per-Device Mouse & Trackball Controls:** Target pointing device selector (`Logitech G Pro (Mouse)`, `Logitech ERGO M575 (Trackball)`, and `All Mice (Global Default)`) with independent pointer speed sliders, flat acceleration toggles, left-handed mode, polling rate (`125Hz`–`1000Hz`), and hardware sensor DPI (`400`–`3200 DPI`) configured live via Niri (`~/.config/niri/amane.kdl`) and `solaar`.
- **Quick Appearance Controls (Utility Panel):**
  - Dedicated **Appearance (`󰏘`)** tab in the Status Bar's Utility overlay (`amane ipc call utility show appearance`).
  - **Colors Subtab:** Mode picker (Auto / Light / Dark), Wallpaper accent toggle, 7 curated accent swatches, and 34 full color schemes in an expandable menu.
  - **Cursor Subtab:** 4 cursor size presets (24px, 32px, 48px, 64px), Shake to Find Cursor toggle, and a picker for 10 popular cursor themes.
  - **Fonts Subtab:** Live typography switcher supporting 10 shell font family choices.
- **Dynamic Theming & Desktop Integrations:**
  - **Alacritty Terminal:** Native configuration generation writing to `~/.config/alacritty/amane-colors.toml` with instant inotify live reloading.
  - **GTK 3 & GTK 4:** Dynamic CSS symlinks (`~/.config/gtk-3.0/gtk.css` and `~/.config/gtk-4.0/gtk.css`), color-scheme preference, and matching GNOME interface accent-color.
  - **Zed Editor:** Custom theme generation at `~/.config/zed/themes/amane.json` with dynamic syntax highlighting and UI token mapping.
  - **COSMIC & Iced Applications:**
    - Generates COSMIC theme builder and palette files in `~/.config/cosmic/` (supporting both v1 and v2 schemas).
    - Features an embedded, native D-Bus `com.system76.CosmicSettingsDaemon` service bridge inside the shell providing `WatchConfig`/`WatchState` and broadcasting `Changed` signals, allowing running Iced/COSMIC apps (`cosmic-files`, `cosmic-edit`, etc.) to reload dynamically on the fly without restarting.
- **Native Monitor Manager & Safety Guard (`src/monitors.rs`):**
  - Integrated Rust background service (`amane::Service`) that monitors Wayland/Niri output states via `event-stream` with 0 extra processes and ~0% CPU.
  - **Undocked Safety Guard:** Automatically detects when the laptop is disconnected from a dock or boots without an external display, guaranteeing `eDP-1` is powered ON at scale `2.0` (`3200x1800@59.982`), preventing black screen lockouts.
  - **Docked Consistency Guard:** Matches external displays by their hardware EDID identity (Make/Model/Serial) rather than transient connector names (`DP-5` vs `DP-1`), preventing USB-C/Thunderbolt dock re-indexing from resetting your scale or layout.
  - **Clamshell Mode Support:** Integrated with Niri's `switch-events` to cleanly power off the laptop display when the lid is closed while docked, and automatically restore it when opened or undocked.
  - **IPC Integration:** `amane ipc call monitors sync` for instant layout synchronization.

## Installation & Running

Clone the repository:

```sh
git clone https://github.com/mystrorc/swifty.git ~/git/Swifty
```

Build the shell:

```sh
cargo build --release
```

Or start it with Niri in `~/.config/niri/config.kdl`:

```kdl
spawn-at-startup "swifty"
```
