//! DataModel for a SpreadSheet
//!
//! # Overview
//!
//! Sheets can contain a [Tbl]. Tbl's contain a collection of [Address] to [Computable]
//! associations. From this we can compute the dimensions of a Tbl as well as render
//! them into a [Table] Widget.

use ratatui::widgets::{Cell, Row, Table};

use std::collections::BTreeMap;

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

/// The computable value located at an [Address].
#[derive(Debug)]
pub enum Computable {
    Text(String),
    Number(f64),
    Formula(String),
}

impl Default for Computable {
    fn default() -> Self {
        Self::Text("".to_owned())
    }
}

/// A single table of addressable computable values.
#[derive(Default, Debug)]
pub struct Tbl {
    addresses: BTreeMap<Address, Computable>,
}

impl Tbl {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn dimensions(&self) -> (usize, usize) {
        let (mut row, mut col) = (0, 0);
        for (addr, _) in &self.addresses {
            row = std::cmp::max(row, addr.row);
            col = std::cmp::max(col, addr.col);
        }
        (row, col)
    }

    pub fn get_computable(&self, row: usize, col: usize) -> Option<&Computable> {
        self.addresses.get(&Address::new(row, col))
    }

    pub fn update_entry(&mut self, address: Address, computable: Computable) {
        // TODO(zaphar): At some point we'll need to store the graph of computation
        // dependencies
        self.addresses.insert(address, computable);
    }
}

impl<'t> From<Tbl> for Table<'t> {
    fn from(value: Tbl) -> Self {
        let (row, col) = value.dimensions();
        let rows = (0..=row)
            .map(|ri| {
                (0..=col)
                    .map(|ci| {
                        match value.get_computable(ri, ci) {
                            // TODO(zaphar): Style information
                            Some(Computable::Text(s)) => Cell::new(format!(" {}", s)),
                            Some(Computable::Number(f)) => Cell::new(format!(" {}", f)),
                            Some(Computable::Formula(_expr)) => Cell::new(format!(" .formula. ")),
                            None => Cell::new(format!(" {}:{} ", ri, ci)),
                        }
                    })
                    .collect::<Row>()
            })
            .collect::<Vec<Row>>();
        Table::default().rows(rows)
    }
}

#[cfg(test)]
mod tests;
