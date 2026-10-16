use std::fs;
use amane::Color;

use super::{config_home, hex, write};
use crate::theme::{self, Theme};

const YELLOW: Color = Color::rgb(0xf9, 0xe2, 0xaf);
const BLUE: Color = Color::rgb(0x89, 0xb4, 0xfa);

pub fn export(theme: &Theme) {
    let zed_dir = format!("{}/zed", config_home());
    let themes_dir = format!("{zed_dir}/themes");

    let appearance = if theme.light { "light" } else { "dark" };

    let harmonize = |seed: Color, amount: f32| {
        let lightness = if theme.light { 0.42 } else { 0.70 };
        theme::retone(theme::mix(seed, theme.accent, amount), lightness, 0.48)
    };

    let yellow = harmonize(YELLOW, 0.22);
    let blue = harmonize(BLUE, 0.30);

    let content = format!(
        r#"{{
  "$schema": "https://zed.dev/schema/themes/v0.2.0.json",
  "author": "Amane",
  "name": "Amane Dynamic",
  "themes": [
    {{
      "appearance": "{appearance}",
      "name": "Amane Dynamic",
      "style": {{
        "accent": "{accent}",
        "background": "{bg}",
        "border": "{border}",
        "border.focused": "{accent}",
        "border.selected": "{accent}",
        "editor.background": "{bg}",
        "editor.foreground": "{fg}",
        "editor.gutter.background": "{bg}",
        "editor.line_number": "{muted}",
        "element.background": "{surface}",
        "element.hover": "{hover}",
        "element.selected": "{sel}",
        "elevated_surface.background": "{surface}",
        "foreground": "{fg}",
        "text.muted": "{muted}",
        "syntax": {{
          "attribute": {{ "color": "{yellow}" }},
          "boolean": {{ "color": "{accent}" }},
          "comment": {{ "color": "{muted}" }},
          "constant": {{ "color": "{yellow}" }},
          "function": {{ "color": "{accent}" }},
          "keyword": {{ "color": "{danger}" }},
          "number": {{ "color": "{yellow}" }},
          "operator": {{ "color": "{fg}" }},
          "punctuation": {{ "color": "{secondary}" }},
          "string": {{ "color": "{success}" }},
          "type": {{ "color": "{blue}" }},
          "variable": {{ "color": "{fg}" }}
        }}
      }}
    }},
    {{
      "appearance": "{appearance}",
      "name": "Settings Manager Nightfox",
      "style": {{
        "accent": "{accent}",
        "background": "{bg}",
        "border": "{border}",
        "border.focused": "{accent}",
        "border.selected": "{accent}",
        "editor.background": "{bg}",
        "editor.foreground": "{fg}",
        "editor.gutter.background": "{bg}",
        "editor.line_number": "{muted}",
        "element.background": "{surface}",
        "element.hover": "{hover}",
        "element.selected": "{sel}",
        "elevated_surface.background": "{surface}",
        "foreground": "{fg}",
        "text.muted": "{muted}",
        "syntax": {{
          "attribute": {{ "color": "{yellow}" }},
          "boolean": {{ "color": "{accent}" }},
          "comment": {{ "color": "{muted}" }},
          "constant": {{ "color": "{yellow}" }},
          "function": {{ "color": "{accent}" }},
          "keyword": {{ "color": "{danger}" }},
          "number": {{ "color": "{yellow}" }},
          "operator": {{ "color": "{fg}" }},
          "punctuation": {{ "color": "{secondary}" }},
          "string": {{ "color": "{success}" }},
          "type": {{ "color": "{blue}" }},
          "variable": {{ "color": "{fg}" }}
        }}
      }}
    }}
  ]
}}
"#,
        appearance = appearance,
        accent = hex(theme.accent),
        bg = hex(theme.background),
        border = hex(theme.border),
        fg = hex(theme.text),
        muted = hex(theme.muted_text),
        secondary = hex(theme.secondary_text),
        surface = hex(theme.surface),
        hover = hex(theme.hover_surface),
        sel = hex(theme.selected_surface),
        yellow = hex(yellow),
        danger = hex(theme.danger),
        success = hex(theme.success),
        blue = hex(blue),
    );

    write(&format!("{themes_dir}/amane.json"), &content);
    write(&format!("{themes_dir}/settings-manager.json"), &content);

    // Update settings.json if needed to use "Amane Dynamic"
    let settings_path = format!("{zed_dir}/settings.json");
    if let Ok(mut settings_text) = fs::read_to_string(&settings_path) {
        if !settings_text.contains("\"theme\": \"Amane Dynamic\"") && !settings_text.contains("\"theme\": \"Settings Manager Nightfox\"") {
            settings_text = settings_text.replace("\"theme\":", "\"// theme\":");
            if let Some(pos) = settings_text.rfind('}') {
                settings_text.insert_str(pos, "  \"theme\": \"Amane Dynamic\",\n");
                let _ = fs::write(&settings_path, settings_text);
            }
        }
    }
}
