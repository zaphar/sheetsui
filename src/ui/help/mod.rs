use crate::ui::render::markdown::Markdown;

pub fn to_widget(topic: &str) -> Markdown {
    match topic {
        "navigate" => Markdown::from_str(include_str!("../../../docs/navigation.md")),
        "edit" => Markdown::from_str(include_str!("../../../docs/edit.md")),
        "command" => Markdown::from_str(include_str!("../../../docs/command.md")),
        "visual" => Markdown::from_str(include_str!("../../../docs/visual.md")),
        _ => Markdown::from_str(include_str!("../../../docs/intro.md")),
    }
}
