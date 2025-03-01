use ratatui::{
    self,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Text,
    widgets::{Block, Paragraph, Widget},
};

pub struct Dialog<'w> {
    content: Text<'w>,
    title: &'w str,
    scroll: (u16, u16),
}

impl<'w> Dialog<'w> {
    pub fn new(content: Text<'w>, title: &'w str) -> Self {
        Self {
            content,
            title,
            scroll: (0, 0),
        }
    }

    pub fn scroll(mut self, scroll: (u16, u16)) -> Self {
        self.scroll = scroll;
        self
    }
}

impl<'w> Widget for Dialog<'w> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        // First find the center of the area.
        let content_width = self.content.width();
        let sidebar_width = (area.width - (content_width as u16) + 2) / 2;
        let [_, dialog_area, _] = Layout::horizontal(vec![
            Constraint::Length(sidebar_width),
            Constraint::Fill(1),
            Constraint::Length(sidebar_width),
        ])
        .areas(area);

        let dialog_block = Block::bordered()
            .title_top(self.title)
            .style(Style::default().on_black());
        let dialog = Paragraph::new(self.content.clone())
            .scroll(self.scroll.clone())
            .block(dialog_block)
            .style(Style::default());
        dialog.render(dialog_area, buf);
    }
}
