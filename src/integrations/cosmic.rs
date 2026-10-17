use amane::Service;
use std::sync::OnceLock;
use zbus::blocking::connection::Builder;
use zbus::blocking::Connection;
use zbus::names::{OwnedWellKnownName, WellKnownName};
use zbus::zvariant::{ObjectPath, OwnedObjectPath};

use super::{config_home, hex, write};
use crate::theme::{self, Theme};

static DAEMON_CONN: OnceLock<Connection> = OnceLock::new();

struct CosmicSettingsDaemon;

#[zbus::interface(name = "com.system76.CosmicSettingsDaemon")]
impl CosmicSettingsDaemon {
    fn watch_config(
        &self,
        _id: &str,
        _version: u64,
    ) -> zbus::fdo::Result<(OwnedObjectPath, OwnedWellKnownName)> {
        Ok((
            ObjectPath::try_from("/com/system76/CosmicSettingsDaemon")
                .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?
                .into(),
            WellKnownName::try_from("com.system76.CosmicSettingsDaemon")
                .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?
                .into(),
        ))
    }

    fn watch_state(
        &self,
        _id: &str,
        _version: u64,
    ) -> zbus::fdo::Result<(OwnedObjectPath, OwnedWellKnownName)> {
        Ok((
            ObjectPath::try_from("/com/system76/CosmicSettingsDaemon")
                .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?
                .into(),
            WellKnownName::try_from("com.system76.CosmicSettingsDaemon")
                .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?
                .into(),
        ))
    }
}

struct CosmicConfig;

#[zbus::interface(name = "com.system76.CosmicSettingsDaemon.Config")]
impl CosmicConfig {
    #[zbus(signal)]
    async fn changed(
        emitter: &zbus::object_server::SignalEmitter<'_>,
        id: &str,
        key: &str,
    ) -> zbus::Result<()>;
}

pub fn start_daemon() {
    let _ = std::thread::spawn(|| {
        let conn = Builder::session()
            .and_then(|b| b.name("com.system76.CosmicSettingsDaemon"))
            .and_then(|b| b.serve_at("/com/system76/CosmicSettingsDaemon", CosmicSettingsDaemon))
            .and_then(|b| b.serve_at("/com/system76/CosmicSettingsDaemon", CosmicConfig))
            .and_then(|b| b.build());

        match conn {
            Ok(c) => {
                let _ = DAEMON_CONN.set(c);
                loop {
                    std::thread::park();
                }
            }
            Err(e) => {
                eprintln!("cosmic settings daemon skipped: {e}");
            }
        }
    });
}

fn emit_changed(id: &str, key: &str) {
    if let Some(conn) = DAEMON_CONN.get() {
        let _ = conn.emit_signal(
            None::<&str>,
            "/com/system76/CosmicSettingsDaemon",
            "com.system76.CosmicSettingsDaemon.Config",
            "Changed",
            &(id, key),
        );
    }
}

pub fn export(theme: &Theme) {
    let cosmic_dir = format!("{}/cosmic", config_home());

    let is_dark_str = if theme.light { "false\n" } else { "true\n" };
    write(&format!("{cosmic_dir}/com.system76.CosmicTheme.Mode/v1/is_dark"), is_dark_str);

    let reduce_transparency = crate::settings::Settings::read().flag("reduce_transparency");
    let (bg_alpha_hex, bg_alpha_float) = if reduce_transparency {
        ("FF", 1.0)
    } else {
        ("D9", 0.85) // 85% opacity for clean, lightweight translucency
    };

    let hex_alpha = |c: amane::Color| format!("{}FF", hex(c));
    let hex_color_v2 = |c: amane::Color| format!("Some(\"{}FF\")\n", hex(c));

    let hex_bg = |c: amane::Color| format!("{}{}", hex(c), bg_alpha_hex);
    let hex_color_v2_bg = |c: amane::Color| format!("Some(\"{}{}\")\n", hex(c), bg_alpha_hex);

    let ron_color_v1 = |c: amane::Color| {
        let r = f32::from(c.red()) / 255.0;
        let g = f32::from(c.green()) / 255.0;
        let b = f32::from(c.blue()) / 255.0;
        format!(
            "Some((\n    red: {:.7},\n    green: {:.7},\n    blue: {:.7},\n    alpha: 1.0,\n))\n",
            r, g, b
        )
    };

    let ron_color_v1_bg = |c: amane::Color| {
        let r = f32::from(c.red()) / 255.0;
        let g = f32::from(c.green()) / 255.0;
        let b = f32::from(c.blue()) / 255.0;
        format!(
            "Some((\n    red: {:.7},\n    green: {:.7},\n    blue: {:.7},\n    alpha: {:.2},\n))\n",
            r, g, b, bg_alpha_float
        )
    };

    let frosted_str = if reduce_transparency { "false\n" } else { "true\n" };
    let blur_str = if reduce_transparency { "None\n" } else { "Medium\n" };

    for mode in ["Dark", "Light"] {
        let active_theme = theme::for_mode(Some(mode == "Light"));

        let builder_v1 = format!("{cosmic_dir}/com.system76.CosmicTheme.{mode}.Builder/v1");
        let builder_v2 = format!("{cosmic_dir}/com.system76.CosmicTheme.{mode}.Builder/v2");

        write(&format!("{builder_v1}/accent"), &ron_color_v1(active_theme.accent));
        write(&format!("{builder_v1}/bg_color"), &ron_color_v1_bg(active_theme.background));
        write(&format!("{builder_v1}/destructive"), &ron_color_v1(active_theme.danger));
        write(&format!("{builder_v1}/neutral_tint"), &ron_color_v1(active_theme.surface));
        write(&format!("{builder_v1}/success"), &ron_color_v1(active_theme.success));
        write(&format!("{builder_v1}/text_tint"), &ron_color_v1(active_theme.text));
        write(&format!("{builder_v1}/warning"), &ron_color_v1(active_theme.accent_hover));
        write(&format!("{builder_v1}/frosted_windows"), frosted_str);
        write(&format!("{builder_v1}/frosted"), blur_str);

        write(&format!("{builder_v2}/accent"), &hex_color_v2(active_theme.accent));
        write(&format!("{builder_v2}/bg_color"), &hex_color_v2_bg(active_theme.background));
        write(&format!("{builder_v2}/destructive"), &hex_color_v2(active_theme.danger));
        write(&format!("{builder_v2}/neutral_tint"), &hex_color_v2(active_theme.surface));
        write(&format!("{builder_v2}/success"), &hex_color_v2(active_theme.success));
        write(&format!("{builder_v2}/text_tint"), &hex_color_v2(active_theme.text));
        write(&format!("{builder_v2}/warning"), &hex_color_v2(active_theme.accent_hover));
        write(&format!("{builder_v2}/frosted_windows"), frosted_str);
        write(&format!("{builder_v2}/frosted"), blur_str);

        let accent_str = hex_alpha(active_theme.accent);
        let bg_str = hex_bg(active_theme.background);
        let text_str = hex_alpha(active_theme.text);
        let surf_str = hex_bg(active_theme.surface);
        let border_str = hex_alpha(active_theme.border);

        let surf_tint_str = if reduce_transparency {
            surf_str.clone()
        } else {
            format!("{}30", hex(active_theme.surface))
        };

        let accent_block = format!(
            "(\n\
             base: \"{accent_str}\",\n\
             hover: \"{accent_str}\",\n\
             pressed: \"{accent_str}\",\n\
             selected: \"{accent_str}\",\n\
             selected_text: \"{accent_str}\",\n\
             focus: \"{accent_str}\",\n\
             divider: \"#000000FF\",\n\
             on: \"#000000FF\",\n\
             disabled: \"{accent_str}\",\n\
             on_disabled: \"{accent_str}\",\n\
             border: \"{accent_str}\",\n\
             disabled_border: \"{accent_str}\",\n\
             )\n"
        );

        let bg_block = format!(
            "Container(\n\
             base: \"{bg_str}\",\n\
             component: (\n\
                 base: \"{surf_str}\",\n\
                 hover: \"{border_str}\",\n\
                 pressed: \"{accent_str}\",\n\
                 selected: \"{border_str}\",\n\
                 selected_text: \"{accent_str}\",\n\
                 focus: \"{accent_str}\",\n\
                 divider: \"{border_str}\",\n\
                 on: \"{text_str}\",\n\
                 disabled: \"{surf_str}\",\n\
                 on_disabled: \"{text_str}\",\n\
                 border: \"{border_str}\",\n\
                 disabled_border: \"{border_str}\",\n\
             ),\n\
             divider: \"{border_str}\",\n\
             on: \"{text_str}\",\n\
             small_widget: \"#3B3B3B40\",\n\
             )\n"
        );

        let transparent_bg_block = format!(
            "Container(\n\
             base: \"{bg_str}\",\n\
             component: (\n\
                 base: \"{surf_tint_str}\",\n\
                 hover: \"{border_str}\",\n\
                 pressed: \"{accent_str}\",\n\
                 selected: \"{border_str}\",\n\
                 selected_text: \"{accent_str}\",\n\
                 focus: \"{accent_str}\",\n\
                 divider: \"{border_str}\",\n\
                 on: \"{text_str}\",\n\
                 disabled: \"{surf_tint_str}\",\n\
                 on_disabled: \"{text_str}\",\n\
                 border: \"{border_str}\",\n\
                 disabled_border: \"{border_str}\",\n\
             ),\n\
             divider: \"{border_str}\",\n\
             on: \"{text_str}\",\n\
             small_widget: \"#3B3B3B40\",\n\
             )\n"
        );

        for ver in ["v1", "v2"] {
            let theme_dir = format!("{cosmic_dir}/com.system76.CosmicTheme.{mode}/{ver}");
            write(&format!("{theme_dir}/accent"), &accent_block);
            write(&format!("{theme_dir}/background"), &bg_block);
            write(&format!("{theme_dir}/transparent_background"), &transparent_bg_block);
            write(&format!("{theme_dir}/frosted_windows"), frosted_str);
            write(&format!("{theme_dir}/frosted"), blur_str);
            write(&format!("{theme_dir}/name"), "\"Amane\"\n");
        }
    }

    write(
        &format!("{cosmic_dir}/settings-manager-theme.ron"),
        &format!("(\n    name: \"Amane Dynamic\",\n    accent: \"{}\",\n)\n", hex(theme.accent)),
    );

    // Broadcast change signals to all subscribed libcosmic / iced apps
    emit_changed("com.system76.CosmicTheme.Mode", "is_dark");
    for mode in ["Dark", "Light"] {
        emit_changed(&format!("com.system76.CosmicTheme.{mode}"), "accent");
        emit_changed(&format!("com.system76.CosmicTheme.{mode}"), "background");
        emit_changed(&format!("com.system76.CosmicTheme.{mode}"), "transparent_background");
        emit_changed(&format!("com.system76.CosmicTheme.{mode}"), "frosted_windows");
        emit_changed(&format!("com.system76.CosmicTheme.{mode}"), "frosted");
        emit_changed(&format!("com.system76.CosmicTheme.{mode}"), "primary");
        emit_changed(&format!("com.system76.CosmicTheme.{mode}"), "name");
        emit_changed(&format!("com.system76.CosmicTheme.{mode}.Builder"), "accent");
        emit_changed(&format!("com.system76.CosmicTheme.{mode}.Builder"), "bg_color");
        emit_changed(&format!("com.system76.CosmicTheme.{mode}.Builder"), "frosted_windows");
        emit_changed(&format!("com.system76.CosmicTheme.{mode}.Builder"), "frosted");
    }
}
