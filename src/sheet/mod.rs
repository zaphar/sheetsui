//! DataModel for a SpreadSheet
//!
//! # Overview
//!
//! Sheets can contain a [Tbl]. Tbl's contain a collection of [Address] to [Computable]
//! associations. From this we can compute the dimensions of a Tbl as well as render
//! them into a [Table] Widget.

use anyhow::{anyhow, Result};
use csvx;
use ironcalc::base::{Workbook, Table};

use std::borrow::Borrow;

/// The Address in a [Tbl].
#[derive(Default, Debug, PartialEq, PartialOrd, Ord, Eq, Clone)]
pub struct Address {
    pub row: usize,
    pub col: usize,
}

impl Address {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

/// A single table of addressable computable values.
pub struct Tbl {
    pub csv: csvx::Table,
    pub location: Address,
}

impl Tbl {
    pub fn new() -> Self {
        Self::from_str("").unwrap()
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
            location: Address::default(),
        })
    }

    pub fn get_raw_value(&self, Address {row, col}: &Address) -> String {
        self.csv.get_raw_table()[*row][*col].clone()
    }

    pub fn move_to(&mut self, addr: Address) -> Result<()> {
        let (row, col) = self.dimensions();
        if addr.row >= row || addr.col >= col {
            return Err(anyhow!("Invalid address to move to: {:?}", addr));
        }
        self.location = addr;
        Ok(())
    }

    pub fn update_entry(&mut self, address: &Address, value: String) -> Result<()> {
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
            .update(address.col, address.row, value.trim())?)
    }
}

#[cfg(test)]
mod tests;
