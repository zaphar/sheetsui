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

/// A spreadsheet book with some internal state tracking.
pub struct Book {
    pub(crate) model: Model,
    pub current_sheet: u32,
    pub location: crate::ui::Address,
    // TODO(zaphar): Because the ironcalc model is sparse we need to track our render size
    // separately
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

    /// Get the sheet data for the current worksheet.
    pub fn get_sheet_data(&self) -> Result<&SheetData> {
        Ok(&self.get_sheet()?.sheet_data)
    }

    /// Move to a specific sheel location in the current sheet
    pub fn move_to(&mut self, Address { row, col }: &Address) -> Result<()> {
        // FIXME(zaphar): Check that this is safe first.
        self.location.row = *row;
        self.location.col = *col;
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
        self.update_entry(&self.location.clone(), value)?;
        Ok(())
    }

    /// Update an entry in the current sheet for a book.
    /// This update won't be reflected until you call `Book::evaluate`.
    pub fn update_entry<S: Into<String>>(&mut self, location: &Address, value: S) -> Result<()> {
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
        if let Some(sheet) = self
            .model
            .workbook
            .worksheets
            .iter()
            .find(|sheet| sheet.name == name)
        {
            self.current_sheet = sheet.sheet_id;
            return true;
        }
        false
    }

    /// Get all sheet names
    pub fn get_sheet_names(&self) -> Vec<String> {
        self.model.workbook.get_worksheet_names()
    }

    /// Select a sheet by id.
    pub fn select_sheet_by_id(&mut self, id: u32) -> bool {
        if let Some(sheet) = self
            .model
            .workbook
            .worksheets
            .iter()
            .find(|sheet| sheet.sheet_id == id)
        {
            self.current_sheet = sheet.sheet_id;
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
            .map_err(|s| anyhow!("Invalid Worksheet: {}", s))?)
    }
}

impl Default for Book {
    fn default() -> Self {
        let mut book =
            Book::new(Model::new_empty("default_name", "en", "America/New_York").unwrap());
        book.update_entry(&Address { row: 1, col: 1 }, "").unwrap();
        book
    }
}
