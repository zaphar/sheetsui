use anyhow::Result;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Rect},
    style::{Color, Stylize},
    text::Text,
    widgets::{Block, Cell, Row, StatefulWidget, Table, Widget},
};

use super::{Address, Book};

// TODO(zaphar): Move this to the book module.
// NOTE(zaphar): This is stolen from ironcalc but ironcalc doesn't expose it
// publically.
pub(crate) const LAST_COLUMN: usize = 16_384;
pub(crate) const LAST_ROW: usize = 1_048_576;

/// A visible column to show in our Viewport.
#[derive(Clone, Debug)]
pub struct VisibleColumn {
    pub idx: usize,
    pub length: u16,
}

impl<'a> From<&'a VisibleColumn> for Constraint {
    fn from(value: &'a VisibleColumn) -> Self {
        Constraint::Length(value.length as u16)
    }
}

#[derive(Debug, Default)]
pub struct ViewportState {
    prev_corner: Address,
}

/// A renderable viewport over a book.
pub struct Viewport<'book> {
    pub(crate) selected: Address,
    book: &'book Book,
    block: Option<Block<'book>>,
}

pub(crate) const COLNAMES: [&'static str; 26] = [
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S",
    "T", "U", "V", "W", "X", "Y", "Z",
];

impl<'book> Viewport<'book> {
    pub fn new(book: &'book Book) -> Self {
        Self {
            book,
            selected: Default::default(),
            block: None,
        }
    }

    pub fn with_selected(mut self, location: Address) -> Self {
        self.selected = location;
        self
    }

    pub(crate) fn get_visible_rows(&self, height: u16, state: &ViewportState) -> Vec<usize> {
        // NOTE(jeremy): For now the row default height is 1. We'll have
        // to adjust that if this changes.
        let mut length = 1;
        let start_row = std::cmp::min(self.selected.row, state.prev_corner.row);
        let mut start = start_row;
        let mut end = start_row;
        for row_idx in start_row..=LAST_ROW {
            let updated_length = length + 1;
            if updated_length <= height {
                length = updated_length;
                end = row_idx;
            } else if self.selected.row >= row_idx {
                start = start + 1;
                end = row_idx;
            } else {
                //dbg!(&start);
                //dbg!(&end);
                break;
            }
        }
        return (start..=end).collect();
    }

    pub(crate) fn get_visible_columns(
        &self,
        width: u16,
        state: &ViewportState,
    ) -> Result<Vec<VisibleColumn>> {
        let mut visible = Vec::new();
        // TODO(zaphar): This should be a shared constant with our first column.
        // We start out with a length of 5 already reserved
        let mut length = 5;
        let start_idx = std::cmp::min(self.selected.col, state.prev_corner.col);
        for idx in start_idx..=LAST_COLUMN {
            let size = self.book.get_col_size(idx)? as u16;
            let updated_length = length + size;
            let col = VisibleColumn { idx, length: size };
            if updated_length < width {
                length = updated_length;
                visible.push(col);
            } else if self.selected.col >= col.idx {
                // We need a sliding window now
                if let Some(first) = visible.first() {
                    // subtract the first columns size.
                    length = length - first.length;
                    // remove the first column.
                    visible = visible.into_iter().skip(1).collect();
                }
                // Add this col to the visible.
                length += size;
                visible.push(col);
                // What if the length is still too long?
                if length > width {
                    if let Some(first) = visible.first() {
                        // subtract the first columns size.
                        length = length - first.length;
                    }
                    visible = visible.into_iter().skip(1).collect();
                }
            } else {
                break;
            }
        }
        return Ok(visible);
    }

    pub fn block(mut self, block: Block<'book>) -> Self {
        self.block = Some(block);
        self
    }

    pub(crate) fn to_table<'widget>(
        &self,
        width: u16,
        height: u16,
        state: &mut ViewportState,
    ) -> Result<Table<'widget>> {
        let visible_columns = self.get_visible_columns(width, state)?;
        let visible_rows = self.get_visible_rows(height, state);
        if let Some(vc) = visible_columns.first() {
            state.prev_corner.col = vc.idx
        }
        if let Some(vr) = visible_rows.first() {
            state.prev_corner.row = *vr;
        }
        let rows: Vec<Row> =
            visible_rows
                .into_iter()
                .map(|ri| {
                    let mut cells = vec![Cell::new(Text::from(ri.to_string()))];
                    cells.extend(visible_columns.iter().map(
                        |VisibleColumn { idx: ci, length: _ }| {
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
                        },
                    ));
                    Row::new(cells)
                })
                .collect();
        let constraints: Vec<Constraint> = visible_columns
            .iter()
            .map(|vc| Constraint::from(vc))
            .collect();
        let end_idx = visible_columns.last().unwrap().idx;
        let mut header = Vec::with_capacity(constraints.len());
        header.push(Cell::new(""));
        header.extend((state.prev_corner.col..end_idx).map(|i| {
            let count = if i == 26 { 1 } else { (i / 26) + 1 };
            Cell::new(COLNAMES[(i - 1) % 26].repeat(count))
        }));
        let mut col_constraints = vec![Constraint::Length(5)];
        col_constraints.extend(constraints.into_iter());
        Ok(Table::new(rows, col_constraints)
            .header(Row::new(header).underlined())
            .column_spacing(1)
            .flex(Flex::Start))
    }
}

impl<'book> StatefulWidget for Viewport<'book> {
    type State = ViewportState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // The block surrounding this table adds 2 additional rows and columns
        // to the available rect for rendering this table.
        let mut table = self
            .to_table(area.width - 2, area.height - 2, state)
            .expect("Failed to turn viewport into a table.");
        if let Some(block) = self.block {
            table = table.block(block);
        }
        Widget::render(table, area, buf);
    }
}
