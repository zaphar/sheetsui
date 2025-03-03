use std::cmp::max;

use anyhow::{anyhow, Result};
use ironcalc::{
    base::{
        expressions::types::Area,
        types::{SheetData, Style, Worksheet},
        worksheet::WorksheetDimension,
        Model, UserModel,
    },
    export::save_xlsx_to_writer,
    import::load_from_xlsx,
};

use crate::ui::Address;

#[cfg(test)]
mod test;

pub(crate) const COL_PIXELS: f64 = 5.0;
// NOTE(zaphar): This is stolen from ironcalc but ironcalc doesn't expose it
// publically.
pub(crate) const LAST_COLUMN: i32 = 16_384;
pub(crate) const LAST_ROW: i32 = 1_048_576;


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
                row.push(Address { sheet: self.start.sheet, row: *ri, col: *ci });
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
                rows.push(Address { sheet: self.start.sheet, row: *ri, col: *ci });
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
    pub(crate) model: UserModel,
    pub location: crate::ui::Address,
    pub dirty: bool,
}

impl Book {
    /// Construct a new book from a Model
    pub fn new(model: UserModel) -> Self {
        Self {
            model,
            location: Address::default(),
            dirty: false,
        }
    }

    pub fn from_model(model: Model) -> Self {
        Self::new(UserModel::from_model(model))
    }

    /// Construct a new book from an xlsx file.
    pub fn new_from_xlsx(path: &str) -> Result<Self> {
        Ok(Self::from_model(load_from_xlsx(
            path,
            "en",
            "America/New_York",
        )?))
    }

    /// Evaluate the spreadsheet calculating formulas and style changes.
    /// This can be an expensive operation.
    pub fn evaluate(&mut self) {
        self.model.evaluate();
    }

    /// Construct a new book from a path.
    pub fn new_from_xlsx_with_locale(path: &str, locale: &str, tz: &str) -> Result<Self> {
        Ok(Self::from_model(load_from_xlsx(path, locale, tz)?))
    }

    /// Save book to an xlsx file.
    pub fn save_to_xlsx(&mut self, path: &str) -> Result<()> {
        // TODO(zaphar): Currently overwrites. Should we prompt in this case?
        let file_path = std::path::Path::new(path);
        let file = std::fs::File::create(file_path)?;
        let writer = std::io::BufWriter::new(file);
        save_xlsx_to_writer(self.model.get_model(), writer)?;
        self.dirty = false;
        Ok(())
    }

    /// Get all the sheet identiers a `Vec<(String, u32)>` where the string
    /// is the sheet name and the u32 is the sheet index.
    pub fn get_all_sheets_identifiers(&self) -> Vec<(String, u32)> {
        self.model
            .get_worksheets_properties()
            .iter()
            .map(|sheet| (sheet.name.to_owned(), sheet.sheet_id))
            .collect()
    }

    /// Get the current sheets name.
    pub fn get_sheet_name(&self) -> Result<&str> {
        Ok(&self.get_sheet()?.name)
    }

    pub fn set_sheet_name(&mut self, idx: u32, sheet_name: &str) -> Result<()> {
        self.model
            .rename_sheet(idx, sheet_name)
            .map_err(|e| anyhow!(e))?;
        self.dirty = true;
        Ok(())
    }

    pub fn new_sheet(&mut self, sheet_name: Option<&str>) -> Result<()> {
        self.model.new_sheet().map_err(|e| anyhow!(e))?;
        let idx = self.model.get_selected_sheet();
        if let Some(name) = sheet_name {
            self.set_sheet_name(idx, name)?;
        }
        self.model
            .set_selected_sheet(self.location.sheet)
            .map_err(|e| anyhow!(e))?;
        self.dirty = true;
        Ok(())
    }

    /// Get the sheet data for the current worksheet.
    pub fn get_sheet_data(&self) -> Result<&SheetData> {
        Ok(&self.get_sheet()?.sheet_data)
    }

    /// Move to a specific sheet location in the current sheet
    pub fn move_to(&mut self, Address { sheet: _, row, col }: &Address) -> Result<()> {
        // FIXME(zaphar): Check that this is safe first.
        self.location.row = *row;
        self.location.col = *col;
        self.dirty = true;
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
                .get_model()
                .extend_to(
                    self.location.sheet,
                    from.row as i32,
                    from.col as i32,
                    cell.row as i32,
                    cell.col as i32,
                )
                .map_err(|e| anyhow!(e))?;
            self.model
                .set_user_input(
                    self.location.sheet,
                    cell.row as i32,
                    cell.col as i32,
                    &contents,
                )
                .map_err(|e| anyhow!(e))?;
        }
        self.evaluate();
        self.dirty = true;
        Ok(())
    }

    pub fn clear_current_cell(&mut self) -> Result<()> {
        self.dirty = true;
        self.clear_cell_contents(self.location.clone())
    }

    pub fn clear_current_cell_all(&mut self) -> Result<()> {
        self.dirty = true;
        self.clear_cell_all(self.location.clone())
    }

    pub fn clear_cell_contents(&mut self, Address { sheet, row, col }: Address) -> Result<()> {
        self.dirty = true;
        Ok(self
            .model
            .range_clear_contents(&Area {
                sheet,
                row: row as i32,
                column: col as i32,
                width: 1,
                height: 1,
            })
            .map_err(|s| anyhow!("Unable to clear cell contents {}", s))?)
    }

    pub fn clear_cell_range(&mut self, start: Address, end: Address) -> Result<()> {
        let area = calculate_area(start.sheet, &start, &end);
        self.model
            .range_clear_contents(&area)
            .map_err(|s| anyhow!("Unable to clear cell contents {}", s))?;
        self.dirty = true;
        Ok(())
    }

    pub fn clear_cell_all(&mut self, Address { sheet, row, col }: Address) -> Result<()> {
        self.dirty = true;
        Ok(self
            .model
            .range_clear_all(&Area {
                sheet,
                row: row as i32,
                column: col as i32,
                width: 1,
                height: 1,
            })
            .map_err(|s| anyhow!("Unable to clear cell contents {}", s))?)
    }

    pub fn clear_cell_range_all(&mut self, start: Address, end: Address) -> Result<()> {
        let area = calculate_area(start.sheet, &start, &end);
        self.model
            .range_clear_all(&area)
            .map_err(|s| anyhow!("Unable to clear cell contents {}", s))?;
        self.dirty = true;
        Ok(())
    }

    /// Get a cells formatted content.
    pub fn get_current_cell_rendered(&self) -> Result<String> {
        Ok(self.get_cell_addr_rendered(&self.location)?)
    }

    pub fn get_cell_style(&self, cell: &Address) -> Option<Style> {
        // TODO(jwall): This is modeled a little weird. We should probably record
        // the error *somewhere* but for the user there is nothing to be done except
        // not use a style.
        match self
            .model
            .get_cell_style(cell.sheet, cell.row as i32, cell.col as i32)
        {
            Err(_) => None,
            Ok(s) => Some(s),
        }
    }

    /// Set the cell style
    /// Valid style paths are:
    /// * fill.bg_color background color
    /// * fill.fg_color foreground color
    /// * font.b bold
    /// * font.i italicize
    /// * font.strike strikethrough
    /// * font.color font color
    /// * num_fmt number format
    /// * alignment turn off alignment
    /// * alignment.horizontal make alignment horzontal
    /// * alignment.vertical make alignment vertical
    /// * alignment.wrap_text wrap cell text
    pub fn set_cell_style(&mut self, style: &[(&str, &str)], area: &Area) -> Result<()> {
        for (path, val) in style {
            self.model
                .update_range_style(area, path, val)
                .map_err(|s| anyhow!("Unable to format cell {}", s))?;
        }
        self.dirty = true;
        Ok(())
    }

    fn get_col_range(&self, sheet: u32, col_idx: usize) -> Area {
        Area {
            sheet,
            row: 1,
            column: col_idx as i32,
            width: 1,
            height: LAST_ROW,
        }
    }

    fn get_row_range(&self, sheet: u32, row_idx: usize) -> Area {
        Area {
            sheet,
            row: row_idx as i32,
            column: 1,
            width: LAST_COLUMN,
            height: 1,
        }
    }

    /// Set the column style.
    /// Valid style paths are:
    /// * fill.bg_color background color
    /// * fill.fg_color foreground color
    /// * font.b bold
    /// * font.i italicize
    /// * font.strike strikethrough
    /// * font.color font color
    /// * num_fmt number format
    /// * alignment turn off alignment
    /// * alignment.horizontal make alignment horzontal
    /// * alignment.vertical make alignment vertical
    /// * alignment.wrap_text wrap cell text
    pub fn set_col_style(
        &mut self,
        style: &[(&str, &str)],
        sheet: u32,
        col_idx: usize,
    ) -> Result<()> {
        let area = self.get_col_range(sheet, col_idx);
        self.set_cell_style(style, &area)?;
        self.dirty = true;
        Ok(())
    }

    /// Set the row style
    /// Valid style paths are:
    /// * fill.bg_color background color
    /// * fill.fg_color foreground color
    /// * font.b bold
    /// * font.i italicize
    /// * font.strike strikethrough
    /// * font.color font color
    /// * num_fmt number format
    /// * alignment turn off alignment
    /// * alignment.horizontal make alignment horzontal
    /// * alignment.vertical make alignment vertical
    /// * alignment.wrap_text wrap cell text
    pub fn set_row_style(
        &mut self,
        style: &[(&str, &str)],
        sheet: u32,
        row_idx: usize,
    ) -> Result<()> {
        let area = self.get_row_range(sheet, row_idx);
        self.set_cell_style(style, &area)?;
        self.dirty = true;
        Ok(())
    }

    /// Get a cells rendered content for display.
    pub fn get_cell_addr_rendered(&self, Address { sheet, row, col }: &Address) -> Result<String> {
        Ok(self
            .model
            .get_formatted_cell_value(*sheet, *row as i32, *col as i32)
            .map_err(|s| anyhow!("Unable to format cell {}", s))?)
    }

    /// Get a cells actual content unformatted as a string.
    pub fn get_cell_addr_contents(&self, Address { sheet, row, col }: &Address) -> Result<String> {
        Ok(self
            .model
            .get_cell_content(*sheet, *row as i32, *col as i32)
            .map_err(|s| anyhow!("Unable to format cell {}", s))?)
    }

    /// Get a cells actual content as a string.
    pub fn get_current_cell_contents(&self) -> Result<String> {
        Ok(self
            .model
            .get_cell_content(
                self.location.sheet,
                self.location.row as i32,
                self.location.col as i32,
            )
            .map_err(|s| anyhow!("Unable to format cell {}", s))?)
    }

    /// Update the current cell in a book.
    /// This update won't be reflected until you call `Book::evaluate`.
    pub fn edit_current_cell<S: AsRef<str>>(&mut self, value: S) -> Result<()> {
        self.dirty = true;
        self.update_cell(&self.location.clone(), value)?;
        Ok(())
    }

    /// Update an entry in the current sheet for a book.
    /// This update won't be reflected until you call `Book::evaluate`.
    pub fn update_cell<S: AsRef<str>>(&mut self, location: &Address, value: S) -> Result<()> {
        self.model
            .set_user_input(
                location.sheet,
                location.row as i32,
                location.col as i32,
                // TODO(jwall): This could probably be made more efficient
                value.as_ref(),
            )
            .map_err(|e| anyhow!("Invalid cell contents: {}", e))?;
        self.dirty = true;
        Ok(())
    }

    /// Insert `count` rows at a `row_idx`.
    pub fn insert_rows(&mut self, row_idx: usize, count: usize) -> Result<()> {
        for i in 0..count {
            self.model
                .insert_row(self.location.sheet, (row_idx + i) as i32)
                .map_err(|e| anyhow!("Unable to insert row(s): {}", e))?;
        }
        if self.location.row >= row_idx {
            self.move_to(&Address {
                sheet: self.location.sheet,
                row: self.location.row + count,
                col: self.location.col,
            })?;
        }
        self.dirty = true;
        Ok(())
    }

    /// Insert `count` columns at a `col_idx`.
    pub fn insert_columns(&mut self, col_idx: usize, count: usize) -> Result<()> {
        for i in 0..count {
            self.model
                .insert_column(self.location.sheet, (col_idx + i) as i32)
                .map_err(|e| anyhow!("Unable to insert column(s): {}", e))?;
        }
        if self.location.col >= col_idx {
            self.move_to(&Address {
                sheet: self.location.sheet,
                row: self.location.row,
                col: self.location.col + count,
            })?;
        }
        self.dirty = true;
        Ok(())
    }

    /// Get the current sheets dimensions. This is a somewhat expensive calculation.
    pub fn get_dimensions(&self) -> Result<WorksheetDimension> {
        Ok(self.get_sheet()?.dimension())
    }

    /// Get column size
    pub fn get_col_size(&self, idx: usize) -> Result<usize> {
        self.get_column_size_for_sheet(self.location.sheet, idx)
    }

    pub fn get_column_size_for_sheet(
        &self,
        sheet: u32,
        idx: usize,
    ) -> std::result::Result<usize, anyhow::Error> {
        Ok((self
            .model
            .get_column_width(sheet, idx as i32)
            .map_err(|e| anyhow!("Error getting column width: {:?}", e))?
            / COL_PIXELS) as usize)
    }

    pub fn set_col_size(&mut self, col: usize, width: usize) -> Result<()> {
        self.set_column_size_for_sheet(self.location.sheet, col, width)
    }

    pub fn set_column_size_for_sheet(
        &mut self,
        sheet: u32,
        col: usize,
        width: usize,
    ) -> std::result::Result<(), anyhow::Error> {
        self.model
            .set_column_width(sheet, col as i32, width as f64 * COL_PIXELS)
            .map_err(|e| anyhow!("Error setting column width: {:?}", e))?;
        self.dirty = true;
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
            .get_model()
            .workbook
            .worksheets
            .iter()
            .enumerate()
            .find(|(_idx, sheet)| sheet.name == name)
        {
            self.location.sheet = idx as u32;
            return true;
        }
        false
    }

    /// Get all sheet names
    pub fn get_sheet_names(&self) -> Vec<String> {
        self.model.get_model().workbook.get_worksheet_names()
    }

    pub fn select_next_sheet(&mut self) {
        let len = self.model.get_model().workbook.worksheets.len() as u32;
        let mut next = self.location.sheet + 1;
        if next == len {
            next = 0;
        }
        self.model
            .set_selected_sheet(next)
            .expect("Unexpected error selecting sheet");
        self.location.sheet = next;
    }

    pub fn select_prev_sheet(&mut self) {
        let len = self.model.get_model().workbook.worksheets.len() as u32;
        let next = if self.location.sheet == 0 {
            len - 1
        } else {
            self.location.sheet - 1
        };
        self.model
            .set_selected_sheet(next)
            .expect("Unexpected error selecting sheet");
        self.location.sheet = next;
    }

    /// Select a sheet by id.
    pub fn select_sheet_by_id(&mut self, id: u32) -> bool {
        if let Some((idx, _sheet)) = self
            .model
            .get_model()
            .workbook
            .worksheets
            .iter()
            .enumerate()
            .find(|(_idx, sheet)| sheet.sheet_id == id)
        {
            self.model
                .set_selected_sheet(idx as u32)
                .expect("Unexpected error selecting sheet");
            self.location.sheet = idx as u32;
            return true;
        }
        false
    }

    /// Get the current `Worksheet`.
    pub(crate) fn get_sheet(&self) -> Result<&Worksheet> {
        // TODO(jwall): Is there a cleaner way to do this with UserModel?
        // Looks like it should be done with:
        // https://docs.rs/ironcalc_base/latest/ironcalc_base/struct.UserModel.html#method.get_worksheets_properties
        Ok(self
            .model
            .get_model()
            .workbook
            .worksheet(self.location.sheet)
            .map_err(|s| anyhow!("Invalid Worksheet id: {}: error: {}", self.location.sheet, s))?)
    }

    pub(crate) fn get_sheet_name_by_idx(&self, idx: usize) -> Result<&str> {
        // TODO(jwall): Is there a cleaner way to do this with UserModel?
        // Looks like it should be done with:
        // https://docs.rs/ironcalc_base/latest/ironcalc_base/struct.UserModel.html#method.get_worksheets_properties
        Ok(&self
            .model
            .get_model()
            .workbook
            .worksheet(idx as u32)
            .map_err(|s| anyhow!("Invalid Worksheet: {}", s))?
            .name)
    }
}

fn calculate_area(sheet: u32, start: &Address, end: &Address) -> Area {
    let area = Area {
        sheet,
        row: start.row as i32,
        column: start.col as i32,
        height: (end.row - start.row + 1) as i32,
        width: (end.col - start.col + 1) as i32,
    };
    area
}

impl Default for Book {
    fn default() -> Self {
        let mut book =
            Book::new(UserModel::new_empty("default_name", "en", "America/New_York").unwrap());
        book.update_cell(&Address { sheet: 0, row: 1, col: 1 }, "").unwrap();
        book
    }
}
