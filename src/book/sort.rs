//! Sorting the rows of a sheet.
//!
//! ironcalc has no action that sorts data in place, and moving rows one at a
//! time with `move_rows_action` is quadratic. So a sort reads every cell of
//! the sorted rows and writes them back in the new order. Formulas are
//! translated the way a copy translates them. Cell styles move with their
//! cells; row height and row-level style stay with the row position.
use std::cmp::Ordering;

use anyhow::{anyhow, Result};
use ironcalc::base::{
    cell::CellValue,
    expressions::{
        types::Area,
        utils::{number_to_column, parse_reference_a1},
    },
    types::{Cell, CellType, Style},
};

use super::Book;
use crate::ui::Address;

/// One sort key: a column and its direction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortKey {
    pub col: usize,
    pub ascending: bool,
}

/// The computed value of a key cell, in Excel sort order:
/// numbers < text < booleans < errors. Empty cells are always last.
#[derive(Debug, Clone, PartialEq)]
enum KeyValue {
    Number(f64),
    /// Lowercased, so text compares without case.
    Text(String),
    Boolean(bool),
    Error,
    Empty,
}

impl KeyValue {
    fn rank(&self) -> u8 {
        match self {
            KeyValue::Number(_) => 0,
            KeyValue::Text(_) => 1,
            KeyValue::Boolean(_) => 2,
            KeyValue::Error => 3,
            KeyValue::Empty => 4,
        }
    }

    fn compare(&self, other: &Self, ascending: bool) -> Ordering {
        // Empty cells go last in both directions.
        match (self, other) {
            (KeyValue::Empty, KeyValue::Empty) => return Ordering::Equal,
            (KeyValue::Empty, _) => return Ordering::Greater,
            (_, KeyValue::Empty) => return Ordering::Less,
            _ => {}
        }
        let ord = match (self, other) {
            (KeyValue::Number(a), KeyValue::Number(b)) => {
                a.partial_cmp(b).unwrap_or(Ordering::Equal)
            }
            (KeyValue::Text(a), KeyValue::Text(b)) => a.cmp(b),
            (KeyValue::Boolean(a), KeyValue::Boolean(b)) => a.cmp(b),
            _ => self.rank().cmp(&other.rank()),
        };
        if ascending {
            ord
        } else {
            ord.reverse()
        }
    }
}

/// One cell's content and style, read before any write.
struct CellCopy {
    content: String,
    is_formula: bool,
    style: Style,
}

fn column_name(col: usize) -> String {
    number_to_column(col as i32).unwrap_or_else(|| col.to_string())
}

impl Book {
    /// Sort rows `first_row..=last_row` of `sheet` by `keys`, in order.
    ///
    /// Every cell of each row moves, from column A to the last used column.
    /// The sort is stable. Nothing changes if it returns an error.
    pub fn sort_rows(
        &mut self,
        sheet: u32,
        first_row: usize,
        last_row: usize,
        keys: &[SortKey],
    ) -> Result<()> {
        if keys.is_empty() {
            return Err(anyhow!("No sort key given."));
        }
        for (i, key) in keys.iter().enumerate() {
            if keys[..i].iter().any(|k| k.col == key.col) {
                return Err(anyhow!(
                    "Column {} is given more than one time.",
                    column_name(key.col)
                ));
            }
        }
        if last_row <= first_row {
            return Ok(());
        }
        let (first, last) = (first_row as i32, last_row as i32);
        let width = self.check_sortable(sheet, first, last)?;
        if width == 0 {
            return Ok(());
        }

        let row_keys = (first..=last)
            .map(|row| {
                keys.iter()
                    .map(|k| self.key_value(sheet, row, k.col as i32))
                    .collect::<Result<Vec<_>>>()
            })
            .collect::<Result<Vec<_>>>()?;
        let mut order: Vec<usize> = (0..row_keys.len()).collect();
        // `sort_by` is stable, so rows with equal keys keep their order.
        order.sort_by(|&a, &b| {
            keys.iter()
                .enumerate()
                .map(|(i, k)| row_keys[a][i].compare(&row_keys[b][i], k.ascending))
                .find(|o| o.is_ne())
                .unwrap_or(Ordering::Equal)
        });
        if order.iter().enumerate().all(|(i, &src)| i == src) {
            return Ok(());
        }

        let copies = self.read_rows(sheet, first, &order, width)?;
        self.model.pause_evaluation();
        let result = self.as_action(|book| book.write_rows(sheet, first, width, copies));
        self.model.resume_evaluation();
        result?;
        self.evaluate();
        self.dirty = true;
        Ok(())
    }

    /// Return the last used column of `sheet`, or an error if rows
    /// `first..=last` hold something a rewrite would break.
    fn check_sortable(&self, sheet: u32, first: i32, last: i32) -> Result<i32> {
        let worksheet = self
            .model
            .get_model()
            .workbook
            .worksheet(sheet)
            .map_err(|e| anyhow!(e))?;
        for merged in &worksheet.merge_cells {
            let mut ends = merged.split(':').filter_map(parse_reference_a1);
            if let (Some(start), Some(end)) = (ends.next(), ends.next()) {
                if start.row <= last && end.row >= first {
                    return Err(anyhow!("Cannot sort a range that contains merged cells."));
                }
            }
        }
        let mut width = 0;
        for (row, cols) in &worksheet.sheet_data {
            for (col, cell) in cols {
                width = width.max(*col);
                if (first..=last).contains(row)
                    && matches!(cell, Cell::ArrayFormula { .. } | Cell::SpillCell { .. })
                {
                    return Err(anyhow!("Cannot sort a range that contains array formulas."));
                }
            }
        }
        Ok(width)
    }

    fn key_value(&self, sheet: u32, row: i32, col: i32) -> Result<KeyValue> {
        let model = self.model.get_model();
        let value = model
            .get_cell_value_by_index(sheet, row, col)
            .map_err(|e| anyhow!(e))?;
        Ok(match value {
            CellValue::None => KeyValue::Empty,
            CellValue::Number(n) => KeyValue::Number(n),
            CellValue::Boolean(b) => KeyValue::Boolean(b),
            // ironcalc reports error values as strings.
            CellValue::String(s) => {
                if model
                    .get_cell_type(sheet, row, col)
                    .map_err(|e| anyhow!(e))?
                    == CellType::ErrorValue
                {
                    KeyValue::Error
                } else {
                    KeyValue::Text(s.to_lowercase())
                }
            }
        })
    }

    /// Read every cell of the rows in `order`, as each will be at its target
    /// row. Row `first + i` gets the cells of row `first + order[i]`.
    fn read_rows(
        &self,
        sheet: u32,
        first: i32,
        order: &[usize],
        width: i32,
    ) -> Result<Vec<Vec<CellCopy>>> {
        let model = self.model.get_model();
        let worksheet = model.workbook.worksheet(sheet).map_err(|e| anyhow!(e))?;
        order
            .iter()
            .enumerate()
            .map(|(i, &src)| {
                let (src, target) = (first + src as i32, first + i as i32);
                (1..=width)
                    .map(|col| {
                        Ok(CellCopy {
                            content: model
                                .extend_to(sheet, src, col, target, col)
                                .map_err(|e| anyhow!(e))?,
                            is_formula: worksheet
                                .cell(src, col)
                                .and_then(|c| c.get_formula())
                                .is_some(),
                            style: self
                                .model
                                .get_cell_style(sheet, src, col)
                                .map_err(|e| anyhow!(e))?,
                        })
                    })
                    .collect()
            })
            .collect()
    }

    fn write_rows(
        &mut self,
        sheet: u32,
        first: i32,
        width: i32,
        copies: Vec<Vec<CellCopy>>,
    ) -> Result<()> {
        let height = copies.len() as i32;
        // on_paste_styles writes to ironcalc's selected range. Select it
        // before any write, so a failure here changes nothing.
        self.model
            .set_selected_sheet(sheet)
            .map_err(|e| anyhow!(e))?;
        self.model
            .set_selected_cell(first, 1)
            .map_err(|e| anyhow!(e))?;
        self.model
            .set_selected_range(first, 1, first + height - 1, width)
            .map_err(|e| anyhow!(e))?;
        self.change(|m| {
            m.range_clear_contents(&Area {
                sheet,
                row: first,
                column: 1,
                width,
                height,
            })
        })
        .map_err(|e| anyhow!(e))?;
        let mut styles = Vec::with_capacity(copies.len());
        for (i, row) in copies.into_iter().enumerate() {
            let mut row_styles = Vec::with_capacity(row.len());
            for (j, copy) in row.into_iter().enumerate() {
                if !copy.content.is_empty() {
                    let addr = Address {
                        sheet,
                        row: first as usize + i,
                        col: j + 1,
                    };
                    if copy.is_formula {
                        self.update_cell(&addr, &copy.content)?;
                    } else {
                        self.update_cell_literal(&addr, &copy.content)?;
                    }
                }
                row_styles.push(copy.style);
            }
            styles.push(row_styles);
        }
        self.change(|m| m.on_paste_styles(&styles))
            .map_err(|e| anyhow!(e))?;
        self.model
            .set_selected_sheet(self.location.sheet)
            .map_err(|e| anyhow!(e))?;
        Ok(())
    }
}
