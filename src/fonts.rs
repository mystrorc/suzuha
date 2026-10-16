use amane::Service;

// the family names fontconfig knows
pub const BODY: &str = "Poppins";
pub const NERD: &str = "JetBrainsMono Nerd Font";
pub const SYMBOLS: &str = "Symbols Nerd Font";
pub const MATERIAL: &str = "Material Design Icons";

pub fn body() -> &'static str {
    match crate::settings::Settings::read().text("font_family") {
        "Inter" => "Inter",
        "Adwaita Sans" => "Adwaita Sans",
        "Cantarell" => "Cantarell",
        "JetBrainsMono Nerd Font" | "JetBrains Mono" => "JetBrainsMono Nerd Font",
        "Fira Sans" => "Fira Sans",
        "Roboto" => "Roboto",
        "Ubuntu" => "Ubuntu",
        "Noto Sans" => "Noto Sans",
        "DejaVu Sans" => "DejaVu Sans",
        _ => "Poppins",
    }
}
