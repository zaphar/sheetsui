use anyhow::{anyhow, Result};
use ironcalc::base::{Model, types::{Worksheet, SheetData}};

pub struct Book {
    model: Model,
    current_sheet: u32,
    current_location: (u32, u32),
}

impl Book {
    pub fn get_sheet_name(&self) -> Result<&str> {
        Ok(&self.get_sheet()?.name)
    }

    pub fn get_sheet_data(&self) -> Result<&SheetData> {
        Ok(&self.get_sheet()?.sheet_data)
    }
    
    fn get_sheet(&self) -> Result<&Worksheet> {
        Ok(self.model.workbook.worksheet(self.current_sheet)
            .map_err(|s| anyhow!("Invalid Worksheet: {}", s))?)
    }
    
    fn get_sheet_mut(&mut self) -> Result<&mut Worksheet> {
        Ok(self.model.workbook.worksheet_mut(self.current_sheet)
            .map_err(|s| anyhow!("Invalid Worksheet: {}", s))?)
    }
}
