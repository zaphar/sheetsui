use ratatui::{
    self,
    layout::{Constraint, Flex, Rect},
    style::{Color, Stylize},
    text::{Line, Text},
    widgets::{Block, Cell, Row, Table, Widget},
    Frame,
};
use tui_popup::Popup;

use super::*;

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

const COLNAMES: [&'static str; 26] = [
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S",
    "T", "U", "V", "W", "X", "Y", "Z",
];

impl<'t, 'book: 't> TryFrom<&'book Book> for Table<'t> {
    fn try_from(value: &'book Book) -> std::result::Result<Self, Self::Error> {
        // TODO(zaphar): This is apparently expensive. Maybe we can cache it somehow?
        // We should do the correct thing here if this fails
        let (row_count, col_count) = value.get_size()?;
        let rows: Vec<Row> = (1..=row_count)
            .into_iter()
            .map(|ri| {
                let mut cells = vec![Cell::new(Text::from(ri.to_string()))];
                cells.extend((1..=col_count).into_iter().map(|ci| {
                    // TODO(zaphar): Is this safe?
                    let content = value.get_cell_addr_rendered(&Address{ row: ri, col: ci }).unwrap();
                    let cell = Cell::new(Text::raw(content));
                    match (value.location.row == ri, value.location.col == ci) {
                        (true, true) => cell.fg(Color::White).underlined(),
                        _ => cell
                            .bg(if ri % 2 == 0 {
                                Color::Rgb(57, 61, 71)
                            } else {
                                Color::Rgb(165, 169, 160)
                            })
                            .fg(if ri % 2 == 0 {
                                Color::White
                            } else {
                                Color::Rgb(31, 32, 34)
                            }),
                    }
                    .bold()
                }));
                Row::new(cells)
            })
            .collect();
        let mut constraints: Vec<Constraint> = Vec::new();
        constraints.push(Constraint::Max(5));
        for _ in 0..col_count {
            constraints.push(Constraint::Min(5));
        }
        let mut header = Vec::with_capacity(col_count as usize);
        header.push(Cell::new(""));
        header.extend((0..(col_count as usize)).map(|i| {
            let count = (i / 26) + 1;
            Cell::new(COLNAMES[i % 26].repeat(count))
        }));
        Ok(Table::new(rows, constraints)
            .block(Block::bordered())
            .header(Row::new(header).underlined())
            .column_spacing(1)
            .flex(Flex::SpaceAround))
    }

    type Error = anyhow::Error;
}

pub fn draw(frame: &mut Frame, ws: &mut Workspace) {
    frame.render_widget(ws, frame.area());
}
