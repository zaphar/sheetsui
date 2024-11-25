use std::cmp::min;

use anyhow::Result;
use ratatui::{
    layout::{Constraint, Flex},
    style::{Color, Stylize},
    text::Text,
    widgets::{Block, Cell, Row, Table, Widget},
};

use super::{Address, Book};

// NOTE(jwall): This is stolen from ironcalc but ironcalc doesn't expose it
// publically.
pub(crate) const LAST_COLUMN: usize = 16_384;
//pub(crate) const LAST_ROW: usize = 1_048_576;

/// A visible column to show in our Viewport.
pub struct VisibleColumn {
    pub idx: usize,
    pub length: u16,
}

impl<'a> From<&'a VisibleColumn> for Constraint {
    fn from(value: &'a VisibleColumn) -> Self {
        Constraint::Length(value.length as u16)
    }
}

/// A renderable viewport over a book.
pub struct Viewport<'book> {
    pub(crate) corner: Address,
    book: &'book Book,
    block: Option<Block<'book>>,
}

const COLNAMES: [&'static str; 26] = [
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S",
    "T", "U", "V", "W", "X", "Y", "Z",
];

impl<'book> Viewport<'book> {
    pub fn new(book: &'book Book) -> Self {
        Self {
            book,
            corner: Default::default(),
            block: None,
        }
    }

    pub fn with_corner(mut self, corner: Address) -> Self {
        self.corner = corner;
        self
    }

    fn get_visible_columns(&self, width: u16) -> Result<Vec<VisibleColumn>> {
        let mut visible = Vec::new();
        let mut length = 0;
        for idx in self.corner.col..=LAST_COLUMN {
            let size = self.book.get_col_size(idx)? as u16;
            let updated_length = length + size;
            if updated_length <= width {
                length = updated_length;
                visible.push(VisibleColumn { idx, length: size });
            }
        }
        return Ok(visible);
    }

    pub fn block(mut self, block: Block<'book>) -> Self {
        self.block = Some(block);
        self
    }

    fn to_table<'widget>(&self, width: u16, height: u16) -> Result<Table<'widget>> {
        let visible_columns = self.get_visible_columns(width)?;
        let max_row = min(self.corner.row + height as usize, LAST_COLUMN);
        let rows: Vec<Row> = (self.corner.row..=max_row)
            .into_iter()
            .map(|ri| {
                let mut cells = vec![Cell::new(Text::from(ri.to_string()))];
                cells.extend(visible_columns.iter().map(|VisibleColumn { idx: ci, length: _, }| {
                    // TODO(zaphar): Is this safe?
                    let content = self
                        .book
                        .get_cell_addr_rendered(&Address { row: ri, col: *ci })
                        .unwrap();
                    let cell = Cell::new(Text::raw(content));
                    match (self.book.location.row == ri, self.book.location.col == *ci) {
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
        let constraints: Vec<Constraint> = visible_columns.iter()
            .map(|vc| Constraint::from(vc))
            .collect();
        let mut header = Vec::with_capacity(constraints.len());
        header.push(Cell::new(""));
        header.extend((self.corner.col..constraints.len()).map(|i| {
            let count = (i / 26) + 1;
            Cell::new(COLNAMES[(i-1) % 26].repeat(count))
        }));
        // TODO(zaphar): We should calculate the length from the length of the stringified version of the
        // row indexes.
        let mut col_constraints = vec![Constraint::Length(5)];
        col_constraints.extend(constraints.into_iter());
        Ok(Table::new(rows, col_constraints)
            .header(Row::new(header).underlined())
            .column_spacing(1)
            .flex(Flex::Start))
    }
}

impl<'book> Widget for Viewport<'book> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let mut table = self.to_table(area.width, area.height).expect("Failed to turn viewport into a table.");
        if let Some(block) = self.block {
            table = table.block(block);
        }
        table.render(area, buf);
    }
}
