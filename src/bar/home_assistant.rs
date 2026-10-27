use amane::{Button, Center, Pointer, Rectangle, Row, Service, Text, children};
use crate::fonts;
use crate::overlay::home_assistant::HomeAssistantData;
use crate::overlay::Overlay;
use crate::bar::pill;
use crate::theme::Theme;
use amane::Monitor;

pub fn pill(monitor: &Monitor, theme: &Theme) -> Option<Rectangle> {
    let ha = HomeAssistantData::read();
    if !ha.reachable || ha.lights.is_empty() {
        return None;
    }

    let on_count = ha.lights.iter().filter(|l| l.state).count();
    let has_lights_on = on_count > 0;

    let icon_color = if has_lights_on {
        theme.accent
    } else {
        theme.secondary_text
    };

    let icon = Text::new("󰌵")
        .size(13.0)
        .font(fonts::NERD)
        .tight()
        .color(icon_color);

    let monitor_name = monitor.name.clone();

    if has_lights_on {
        let count_str = format!("{on_count}");
        let text_w = pill::text_width(&count_str, 13.0);
        let width = 20.0 + text_w + 18.0;
        let count_label = pill::label(&count_str, 13.0, theme.text);
        let row = Row::new(children![icon, count_label]).gap(5.0).align(Center);

        Some(
            pill::view(width, theme.surface)
                .cursor(Pointer)
                .on_click(move |button| match button {
                    Button::Left => Overlay::toggle_home_assistant(Some(&monitor_name)),
                    Button::Right => HomeAssistantData::turn_off_all_lights(),
                    _ => {}
                })
                .align_child(Center, Center)
                .child(row),
        )
    } else {
        let width = 34.0;
        Some(
            pill::view(width, theme.surface)
                .cursor(Pointer)
                .on_click(move |button| match button {
                    Button::Left => Overlay::toggle_home_assistant(Some(&monitor_name)),
                    _ => {}
                })
                .align_child(Center, Center)
                .child(icon),
        )
    }
}
