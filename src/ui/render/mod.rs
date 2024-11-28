use ratatui::{
    self,
    layout::Rect,
    text::{Line, Text},
    widgets::{Block, Widget},
    Frame,
};
use tui_popup::Popup;

use super::*;

pub mod viewport;
pub use viewport::Viewport;

#[cfg(test)]
mod test;

impl<'widget, 'ws: 'widget> Widget for &'widget mut Workspace<'ws> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let outer_block = Block::bordered()
            .title(Line::from(
                self.name
                    .file_name()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| String::from("Unknown")),
            ))
            .title_bottom(match self.state.modality() {
                Modality::Navigate => "navigate",
                Modality::CellEdit => "edit",
                Modality::Command => "command",
                Modality::Dialog => "",
            })
            .title_bottom(
                Line::from(format!(
                    "{},{}",
                    self.book.location.row, self.book.location.col
                ))
                .right_aligned(),
            );

        for (rect, f) in self.get_render_parts(area.clone()) {
            f(rect, buf, self);
        }

        outer_block.render(area, buf);

        if self.state.modality() == &Modality::Dialog {
            let lines = Text::from_iter(self.state.popup.iter().cloned());
            let popup = Popup::new(lines);
            popup.render(area, buf);
        }
    }
}

pub fn draw(frame: &mut Frame, ws: &mut Workspace) {
    frame.render_widget(ws, frame.area());
}
