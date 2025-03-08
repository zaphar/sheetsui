use ratatui::{
    self,
    layout::Rect,
    text::{Line, Text},
    widgets::{Block, Paragraph, Tabs, Widget},
    Frame,
};

use super::*;

pub mod viewport;
pub use viewport::Viewport;
pub mod dialog;
pub mod markdown;

#[cfg(test)]
mod test;

impl<'ws> Workspace<'ws> {
    fn get_render_parts(
        &mut self,
        area: Rect,
    ) -> Vec<(Rect, Box<dyn Fn(Rect, &mut Buffer, &mut Self)>)> {
        use ratatui::widgets::StatefulWidget;
        let mut cs = vec![
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Fill(1),
        ];
        let mut rs: Vec<Box<dyn Fn(Rect, &mut Buffer, &mut Self)>> = vec![
            Box::new(|rect: Rect, buf: &mut Buffer, ws: &mut Self| {
                let tabs = Tabs::new(
                    ws.book
                        .get_sheet_names()
                        .iter()
                        .enumerate()
                        .map(|(idx, name)| format!("{} {}", name, idx))
                        .collect::<Vec<String>>(),
                )
                .select(Some(ws.book.location.sheet as usize));
                tabs.render(rect, buf);
            }),
            Box::new(|rect: Rect, buf: &mut Buffer, ws: &mut Self| {
                let [text_rect, info_rect] =
                    Layout::horizontal(vec![Constraint::Fill(1), Constraint::Fill(1)]).areas(rect);
                ws.text_area.render(text_rect, buf);
                let hint = Paragraph::new(vec![
                    Line::from(""),
                    Line::from("ALT-h to toggle help dialog").centered(),
                ]);
                hint.render(info_rect, buf);
            }),
            Box::new(move |rect: Rect, buf: &mut Buffer, ws: &mut Self| {
                let sheet_name = ws.book.get_sheet_name().unwrap_or("Unknown");
                let table_block = Block::bordered().title_top(sheet_name);
                let viewport = Viewport::new(
                    &ws.book,
                    if ws.state.modality() == &Modality::RangeSelect {
                        Some(&ws.state.range_select)
                    } else {
                        None
                    },
                )
                .with_selected(ws.book.location.clone())
                .block(table_block);
                StatefulWidget::render(viewport, rect, buf, &mut ws.state.viewport_state);
            }),
        ];

        if self.state.modality() == &Modality::Command {
            cs.push(Constraint::Max(1));
            rs.push(Box::new(|rect: Rect, buf: &mut Buffer, ws: &mut Self| {
                StatefulWidget::render(
                    TextPrompt::from("Command"),
                    rect,
                    buf,
                    &mut ws.state.command_state,
                )
            }));
        }
        let rects: Vec<Rect> = Vec::from(
            Layout::vertical(cs)
                .vertical_margin(2)
                .horizontal_margin(2)
                .flex(Flex::Legacy)
                .split(area.clone())
                .as_ref(),
        );
        rects
            .into_iter()
            .zip(rs.into_iter())
            .map(|(rect, f)| (rect, f))
            .collect()
    }
}

impl<'widget, 'ws: 'widget> Widget for &'widget mut Workspace<'ws> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        if self.state.modality() == &Modality::Dialog {
            let lines = Text::from_iter(self.state.popup.iter().cloned());
            let popup = dialog::Dialog::new(lines, "Help").scroll(self.state.dialog_scroll);
            popup.render(area, buf);
        } else if self.state.modality() == &Modality::Quit {
            let popup = dialog::Dialog::new(Text::raw("File is not yet saved. Save it first?"), "Quit")
                .with_bottom_title("Y/N");
            popup.render(area, buf);
        } else {
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
                    Modality::RangeSelect => "range-copy",
                    Modality::Quit => "",
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
        }
    }
}

pub fn draw(frame: &mut Frame, ws: &mut Workspace) {
    frame.render_widget(ws, frame.area());
}
