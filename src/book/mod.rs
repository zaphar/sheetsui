use std::cmp::max;

use anyhow::{anyhow, Result};
use ironcalc::{
    base::{
        types::{SheetData, Worksheet},
        worksheet::WorksheetDimension,
        Model,
    },
    export::save_xlsx_to_writer,
    import::load_from_xlsx,
};

use crate::ui::Address;

#[cfg(test)]
mod test;

const COL_PIXELS: f64 = 5.0;

#[derive(Debug, Clone)]
pub struct AddressRange<'book> {
    pub start: &'book Address,
    pub end: &'book Address,
}

impl<'book> AddressRange<'book> {
    pub fn as_rows(&self) -> Vec<Vec<Address>> {
        let (row_range, col_range) = self.get_ranges();
        let mut rows = Vec::with_capacity(row_range.len());
        for ri in row_range.iter() {
            let mut row = Vec::with_capacity(col_range.len());
            for ci in col_range.iter() {
                row.push(Address { row: *ri, col: *ci });
            }
            rows.push(row);
        }
        rows
    }
    
    pub fn as_series(&self) -> Vec<Address> {
        let (row_range, col_range) = self.get_ranges();
        let mut rows = Vec::with_capacity(row_range.len() * col_range.len());
        for ri in row_range.iter() {
            for ci in col_range.iter() {
                rows.push(Address { row: *ri, col: *ci });
            }
        }
        rows
    }

    fn get_ranges(&self) -> (Vec<usize>, Vec<usize>) {
        let row_range = if self.start.row <= self.end.row {
            (self.start.row..=self.end.row)
                .into_iter()
                .collect::<Vec<usize>>()
        } else {
            let mut v = (self.start.row..=self.end.row)
                .into_iter()
                .collect::<Vec<usize>>();
            v.reverse();
            v
        };
        let col_range = if self.start.col <= self.end.col {
            (self.start.col..=self.end.col)
                .into_iter()
                .collect::<Vec<usize>>()
        } else {
            let mut v = (self.start.col..=self.end.col)
                .into_iter()
                .collect::<Vec<usize>>();
            v.reverse();
            v
        };
        (row_range, col_range)
    }
}

/// A spreadsheet book with some internal state tracking.
pub struct Book {
    pub(crate) model: Model,
    pub current_sheet: u32,
    pub location: crate::ui::Address,
}

impl Book {
    /// Construct a new book from a Model
    pub fn new(model: Model) -> Self {
        Self {
            model,
            current_sheet: 0,
            location: Address::default(),
        }
    }

    /// Construct a new book from an xlsx file.
    pub fn new_from_xlsx(path: &str) -> Result<Self> {
        Ok(Self::new(load_from_xlsx(path, "en", "America/New_York")?))
    }

    /// Evaluate the spreadsheet calculating formulas and style changes.
    /// This can be an expensive operation.
    pub fn evaluate(&mut self) {
        self.model.evaluate();
    }

    // TODO(zaphar): Should I support ICalc?
    /// Construct a new book from a path.
    pub fn new_from_xlsx_with_locale(path: &str, locale: &str, tz: &str) -> Result<Self> {
        Ok(Self::new(load_from_xlsx(path, locale, tz)?))
    }

    /// Save book to an xlsx file.
    pub fn save_to_xlsx(&self, path: &str) -> Result<()> {
        // TODO(zaphar): Currently overwrites. Should we prompty in this case?
        let file_path = std::path::Path::new(path);
        let file = std::fs::File::create(file_path)?;
        let writer = std::io::BufWriter::new(file);
        save_xlsx_to_writer(&self.model, writer)?;
        Ok(())
    }

    /// Get all the sheet identiers a `Vec<(String, u32)>` where the string
    /// is the sheet name and the u32 is the sheet index.
    pub fn get_all_sheets_identifiers(&self) -> Vec<(String, u32)> {
        self.model
            .workbook
            .worksheets
            .iter()
            .map(|sheet| (sheet.get_name(), sheet.get_sheet_id()))
            .collect()
    }

    /// Get the current sheets name.
    pub fn get_sheet_name(&self) -> Result<&str> {
        Ok(&self.get_sheet()?.name)
    }

    pub fn set_sheet_name(&mut self, idx: usize, sheet_name: &str) -> Result<()> {
        self.get_sheet_by_idx_mut(idx)?.set_name(sheet_name);
        Ok(())
    }

    pub fn new_sheet(&mut self, sheet_name: Option<&str>) -> Result<()> {
        let (_, idx) = self.model.new_sheet();
        if let Some(name) = sheet_name {
            self.set_sheet_name(idx as usize, name)?;
        }
        Ok(())
    }

    /// Get the sheet data for the current worksheet.
    pub fn get_sheet_data(&self) -> Result<&SheetData> {
        Ok(&self.get_sheet()?.sheet_data)
    }

    /// Move to a specific sheet location in the current sheet
    pub fn move_to(&mut self, Address { row, col }: &Address) -> Result<()> {
        // FIXME(zaphar): Check that this is safe first.
        self.location.row = *row;
        self.location.col = *col;
        Ok(())
    }

    /// Extend a cell to the rest of the range.
    pub fn extend_to(&mut self, from: &Address, to: &Address) -> Result<()> {
        for cell in (AddressRange {
            start: from,
            end: to,
        })
        .as_series()
        .iter()
        .skip(1)
        {
            let contents = self
                .model
                .extend_to(
                    self.current_sheet,
                    from.row as i32,
                    from.col as i32,
                    cell.row as i32,
                    cell.col as i32,
                )
                .map_err(|e| anyhow!(e))?;
            self.model
                .set_user_input(
                    self.current_sheet,
                    cell.row as i32,
                    cell.col as i32,
                    contents,
                )
                .map_err(|e| anyhow!(e))?;
        }
        self.evaluate();
        Ok(())
    }

    pub fn clear_current_cell(&mut self) -> Result<()> {
        self.clear_cell_contents(self.current_sheet as u32, self.location.clone())
    }

    pub fn clear_current_cell_all(&mut self) -> Result<()> {
        self.clear_cell_all(self.current_sheet as u32, self.location.clone())
    }

    pub fn clear_cell_contents(&mut self, sheet: u32, Address { row, col }: Address) -> Result<()> {
        Ok(self
            .model
            .cell_clear_contents(sheet, row as i32, col as i32)
            .map_err(|s| anyhow!("Unable to clear cell contents {}", s))?)
    }

    pub fn clear_cell_range(&mut self, sheet: u32, start: Address, end: Address) -> Result<()> {
        for row in start.row..=end.row {
            for col in start.col..=end.col {
                self.clear_cell_contents(sheet, Address { row, col })?;
            }
        }
        Ok(())
    }

    pub fn clear_cell_all(&mut self, sheet: u32, Address { row, col }: Address) -> Result<()> {
        Ok(self
            .model
            .cell_clear_all(sheet, row as i32, col as i32)
            .map_err(|s| anyhow!("Unable to clear cell contents {}", s))?)
    }

    pub fn clear_cell_range_all(&mut self, sheet: u32, start: Address, end: Address) -> Result<()> {
        for row in start.row..=end.row {
            for col in start.col..=end.col {
                self.clear_cell_all(sheet, Address { row, col })?;
            }
        }
        Ok(())
    }

    /// Get a cells formatted content.
    pub fn get_current_cell_rendered(&self) -> Result<String> {
        Ok(self.get_cell_addr_rendered(&self.location)?)
    }

    /// Get a cells rendered content for display.
    pub fn get_cell_addr_rendered(&self, Address { row, col }: &Address) -> Result<String> {
        Ok(self
            .model
            .get_formatted_cell_value(self.current_sheet, *row as i32, *col as i32)
            .map_err(|s| anyhow!("Unable to format cell {}", s))?)
    }

    /// Get a cells actual content unformatted as a string.
    pub fn get_cell_addr_contents(&self, Address { row, col }: &Address) -> Result<String> {
        Ok(self
            .model
            .get_cell_content(self.current_sheet, *row as i32, *col as i32)
            .map_err(|s| anyhow!("Unable to format cell {}", s))?)
    }

    /// Get a cells actual content as a string.
    pub fn get_current_cell_contents(&self) -> Result<String> {
        Ok(self
            .model
            .get_cell_content(
                self.current_sheet,
                self.location.row as i32,
                self.location.col as i32,
            )
            .map_err(|s| anyhow!("Unable to format cell {}", s))?)
    }

    /// Update the current cell in a book.
    /// This update won't be reflected until you call `Book::evaluate`.
    pub fn edit_current_cell<S: Into<String>>(&mut self, value: S) -> Result<()> {
        self.update_cell(&self.location.clone(), value)?;
        Ok(())
    }

    /// Update an entry in the current sheet for a book.
    /// This update won't be reflected until you call `Book::evaluate`.
    pub fn update_cell<S: Into<String>>(&mut self, location: &Address, value: S) -> Result<()> {
        self.model
            .set_user_input(
                self.current_sheet,
                location.row as i32,
                location.col as i32,
                value.into(),
            )
            .map_err(|e| anyhow!("Invalid cell contents: {}", e))?;
        Ok(())
    }

    /// Insert `count` rows at a `row_idx`.
    pub fn insert_rows(&mut self, row_idx: usize, count: usize) -> Result<()> {
        self.model
            .insert_rows(self.current_sheet, row_idx as i32, count as i32)
            .map_err(|e| anyhow!("Unable to insert row(s): {}", e))?;
        if self.location.row >= row_idx {
            self.move_to(&Address {
                row: self.location.row + count,
                col: self.location.col,
            })?;
        }
        Ok(())
    }

    /// Insert `count` columns at a `col_idx`.
    pub fn insert_columns(&mut self, col_idx: usize, count: usize) -> Result<()> {
        self.model
            .insert_columns(self.current_sheet, col_idx as i32, count as i32)
            .map_err(|e| anyhow!("Unable to insert column(s): {}", e))?;
        if self.location.col >= col_idx {
            self.move_to(&Address {
                row: self.location.row,
                col: self.location.col + count,
            })?;
        }
        Ok(())
    }

    /// Get the current sheets dimensions. This is a somewhat expensive calculation.
    pub fn get_dimensions(&self) -> Result<WorksheetDimension> {
        Ok(self.get_sheet()?.dimension())
    }

    /// Get column size
    pub fn get_col_size(&self, idx: usize) -> Result<usize> {
        Ok((self
            .get_sheet()?
            .get_column_width(idx as i32)
            .map_err(|e| anyhow!("Error getting column width: {:?}", e))?
            / COL_PIXELS) as usize)
    }

    pub fn set_col_size(&mut self, idx: usize, cols: usize) -> Result<()> {
        self.get_sheet_mut()?
            .set_column_width(idx as i32, cols as f64 * COL_PIXELS)
            .map_err(|e| anyhow!("Error setting column width: {:?}", e))?;
        Ok(())
    }

    // Get the size of the current sheet as a `(row_count, column_count)`
    pub fn get_size(&self) -> Result<(usize, usize)> {
        let sheet = &self.get_sheet()?.sheet_data;
        let mut row_count = 0 as i32;
        let mut col_count = 0 as i32;
        for (ri, cols) in sheet.iter() {
            row_count = max(*ri, row_count);
            for (ci, _) in cols.iter() {
                col_count = max(*ci, col_count);
            }
        }
        Ok((row_count as usize, col_count as usize))
    }

    /// Select a sheet by name.
    pub fn select_sheet_by_name(&mut self, name: &str) -> bool {
        if let Some((idx, _sheet)) = self
            .model
            .workbook
            .worksheets
            .iter()
            .enumerate()
            .find(|(_idx, sheet)| sheet.name == name)
        {
            self.current_sheet = idx as u32;
            return true;
        }
        false
    }

    /// Get all sheet names
    pub fn get_sheet_names(&self) -> Vec<String> {
        self.model.workbook.get_worksheet_names()
    }

    pub fn select_next_sheet(&mut self) {
        let len = self.model.workbook.worksheets.len() as u32;
        let mut next = self.current_sheet + 1;
        if next == len {
            next = 0;
        }
        self.current_sheet = next;
    }

    pub fn select_prev_sheet(&mut self) {
        let len = self.model.workbook.worksheets.len() as u32;
        let next = if self.current_sheet == 0 {
            len - 1
        } else {
            self.current_sheet - 1
        };
        self.current_sheet = next;
    }

    /// Select a sheet by id.
    pub fn select_sheet_by_id(&mut self, id: u32) -> bool {
        if let Some((idx, _sheet)) = self
            .model
            .workbook
            .worksheets
            .iter()
            .enumerate()
            .find(|(_idx, sheet)| sheet.sheet_id == id)
        {
            self.current_sheet = idx as u32;
            return true;
        }
        false
    }

    /// Get the current `Worksheet`.
    pub(crate) fn get_sheet(&self) -> Result<&Worksheet> {
        Ok(self
            .model
            .workbook
            .worksheet(self.current_sheet)
            .map_err(|s| anyhow!("Invalid Worksheet id: {}: error: {}", self.current_sheet, s))?)
    }

    pub(crate) fn get_sheet_mut(&mut self) -> Result<&mut Worksheet> {
        Ok(self
            .model
            .workbook
            .worksheet_mut(self.current_sheet)
            .map_err(|s| anyhow!("Invalid Worksheet: {}", s))?)
    }

    pub(crate) fn get_sheet_name_by_idx(&self, idx: usize) -> Result<&str> {
        Ok(&self
            .model
            .workbook
            .worksheet(idx as u32)
            .map_err(|s| anyhow!("Invalid Worksheet: {}", s))?
            .name)
    }
    pub(crate) fn get_sheet_by_idx_mut(&mut self, idx: usize) -> Result<&mut Worksheet> {
        Ok(self
            .model
            .workbook
            .worksheet_mut(idx as u32)
            .map_err(|s| anyhow!("Invalid Worksheet: {}", s))?)
    }
}

impl Default for Book {
    fn default() -> Self {
        let mut book =
            Book::new(Model::new_empty("default_name", "en", "America/New_York").unwrap());
        book.update_cell(&Address { row: 1, col: 1 }, "").unwrap();
        book
    }
}
