use ratatui::text::Text;
use tui_markdown;

pub fn render_topic(topic: &str) -> Text<'static> {
    match topic {
        "navigate" => tui_markdown::from_str(include_str!("../../../docs/navigation.md")),
        "edit" => tui_markdown::from_str(include_str!("../../../docs/edit.md")),
        "command" => tui_markdown::from_str(include_str!("../../../docs/command.md")),
        "visual" => tui_markdown::from_str(include_str!("../../../docs/visual.md")),
        _ => tui_markdown::from_str(include_str!("../../../docs/intro.md")),
    }
}
