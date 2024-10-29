//! DataModel for a SpreadSheet
//!
//! # Overview
//!
//! Sheets can contain a [Tbl]. Tbl's contain a collection of [Address] to [Computable]
//! associations. From this we can compute the dimensions of a Tbl as well as render
//! them into a [Table] Widget.

use anyhow::{anyhow, Result};
use csvx;
use ratatui::widgets::{Cell, Row, Table};

use std::borrow::Borrow;

pub enum CellValue {
    Text(String),
    Float(f64),
    Integer(i64),
    Other(String),
}

impl CellValue {
    pub fn to_csv_value(&self) -> String {
        match self {
            CellValue::Text(v) => format!("\"{}\"", v),
            CellValue::Float(v) => format!("{}", v),
            CellValue::Integer(v) => format!("{}", v),
            CellValue::Other(v) => format!("{}", v),
        }
    }

    pub fn text<S: Into<String>>(value: S) -> CellValue {
        CellValue::Text(Into::<String>::into(value))
    }

    pub fn other<S: Into<String>>(value: S) -> CellValue {
        CellValue::Other(Into::<String>::into(value))
    }

    pub fn float(value: f64) -> CellValue {
        CellValue::Float(value)
    }

    pub fn int(value: i64) -> CellValue {
        CellValue::Integer(value)
    }
}

/// The Address in a [Tbl].
#[derive(Default, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct Address {
    row: usize,
    col: usize,
}

impl Address {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

/// A single table of addressable computable values.
pub struct Tbl {
    csv: csvx::Table,
}

impl Tbl {
    pub fn new() -> Self {
        Self {
            csv: csvx::Table::new("").unwrap(),
        }
    }

    pub fn dimensions(&self) -> (usize, usize) {
        let table = self.csv.get_raw_table();
        let row_count = table.len();
        if row_count > 0 {
            let col_count = table.first().unwrap().len();
            return (row_count, col_count);
        }
        return (0, 0);
    }

    pub fn from_str<S: Borrow<str>>(input: S) -> Result<Self> {
        Ok(Self {
            csv: csvx::Table::new(input)
                .map_err(|e| anyhow!("Error parsing table from csv text: {}", e))?,
        })
    }

    pub fn update_entry(&mut self, address: Address, value: CellValue) -> Result<()> {
        // TODO(zaphar): At some point we'll need to store the graph of computation
        let (row, col) = self.dimensions();
        if address.row >= row {
            // then we need to add rows.
            for r in row..=address.row {
                self.csv.insert_y(r);
            }
        }
        if address.col >= col {
            for c in col..=address.col {
                self.csv.insert_x(c);
            }
        }
        Ok(self
            .csv
            .update(address.col, address.row, value.to_csv_value())?)
    }
}

impl<'t> From<Tbl> for Table<'t> {
    fn from(value: Tbl) -> Self {
        let rows: Vec<Row> = value
            .csv
            .get_calculated_table()
            .iter()
            .map(|r| {
                let cells = r.iter().map(|v| Cell::new(format!("{}", v)));
                Row::new(cells)
            })
            .collect();
        Table::default().rows(rows)
    }
}

#[cfg(test)]
mod tests;
