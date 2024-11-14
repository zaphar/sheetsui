use std::path::PathBuf;

use anyhow::{anyhow, Result};
use ironcalc::{
    base::{
        locale, types::{SheetData, Worksheet}, Model
    },
    export::save_to_xlsx,
    import::load_from_xlsx,
};

/// A spreadsheet book with some internal state tracking.
pub struct Book {
    model: Model,
    current_sheet: u32,
    current_location: (i32, i32),
}

impl Book {

    /// Construct a new book from a Model
    pub fn new(model: Model) -> Self {
        Self {
            model,
            current_sheet: 0,
            current_location: (0, 0),
        }
    }

    // TODO(zaphar): Should I support ICalc?
    /// Construct a new book from a path.
    pub fn new_from_xlsx(path: &str, locale: &str, tz: &str) -> Result<Self> {
        Ok(Self::new(load_from_xlsx(path, locale, tz)?))
    }

    /// Save book to an xlsx file.
    pub fn save_to_xlsx(&self, path: &str) -> Result<()> {
        save_to_xlsx(&self.model, path)?;
        Ok(())
    }

    /// Get the currently set sheets name.
    pub fn get_sheet_name(&self) -> Result<&str> {
        Ok(&self.get_sheet()?.name)
    }

    /// Get the sheet data for the current worksheet.
    pub fn get_sheet_data(&self) -> Result<&SheetData> {
        Ok(&self.get_sheet()?.sheet_data)
    }

    /// Get a cells formatted content.
    pub fn get_cell_rendered(&self) -> Result<String> {
        Ok(self
            .model
            .get_formatted_cell_value(
                self.current_sheet,
                self.current_location.0,
                self.current_location.1,
            )
            .map_err(|s| anyhow!("Unable to format cell {}", s))?)
    }

    /// Get a cells actual content as a string.
    pub fn get_cell_contents(&self) -> Result<String> {
        Ok(self
            .model
            .get_cell_content(
                self.current_sheet,
                self.current_location.0,
                self.current_location.1,
            )
            .map_err(|s| anyhow!("Unable to format cell {}", s))?)
    }

    pub fn edit_cell(&mut self, value: String) -> Result<()> {
        self.model
            .set_user_input(
                self.current_sheet,
                self.current_location.0,
                self.current_location.1,
                value,
            )
            .map_err(|e| anyhow!("Invalid cell contents: {}", e))?;
        Ok(())
    }

    /// Get the current sheets dimensions. This is a somewhat expensive calculation.
    pub fn get_dimensions(&self) -> Result<(usize, usize)> {
        let dimensions = self.get_sheet()?.dimension();
        Ok((
            dimensions.max_row.try_into()?,
            dimensions.max_column.try_into()?,
        ))
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

    fn get_sheet(&self) -> Result<&Worksheet> {
        Ok(self
            .model
            .workbook
            .worksheet(self.current_sheet)
            .map_err(|s| anyhow!("Invalid Worksheet: {}", s))?)
    }

    fn get_sheet_mut(&mut self) -> Result<&mut Worksheet> {
        Ok(self
            .model
            .workbook
            .worksheet_mut(self.current_sheet)
            .map_err(|s| anyhow!("Invalid Worksheet: {}", s))?)
    }
}
