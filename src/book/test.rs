use ironcalc::base::worksheet::WorksheetDimension;

use crate::ui::Address;

use super::Book;

#[test]
fn test_book_default() {
    let mut book = Book::default();
    let WorksheetDimension {
        min_row,
        max_row,
        min_column,
        max_column,
    } = book.get_dimensions().expect("couldn't get dimensions");
    assert_eq!((1, 1, 1, 1), (min_row, max_row, min_column, max_column));
    assert_eq!((1, 1), book.get_size().expect("Failed to get size"));
    let cell = book
        .get_current_cell_contents()
        .expect("couldn't get contents");
    assert_eq!("", cell);
    book.edit_current_cell("1").expect("failed to edit cell");
    book.evaluate();
    let cell = book
        .get_current_cell_contents()
        .expect("couldn't get contents");
    assert_eq!("1", cell);
    let cell = book
        .get_current_cell_rendered()
        .expect("couldn't get contents");
    assert_eq!("1", cell);
    let sheets = book.get_all_sheets_identifiers();
    assert_eq!(1, sheets.len());
}

#[test]
fn test_book_insert_cell_new_row() {
    let mut book = Book::default();
    book.update_cell(
        &Address {
            sheet: 0,
            row: 2,
            col: 1,
        },
        "1",
    )
    .expect("failed to edit cell");
    book.evaluate();
    let WorksheetDimension {
        min_row,
        max_row,
        min_column,
        max_column,
    } = book.get_dimensions().expect("couldn't get dimensions");
    assert_eq!((1, 2, 1, 1), (min_row, max_row, min_column, max_column));
    assert_eq!((2, 1), book.get_size().expect("Failed to get size"));
}

#[test]
fn test_book_insert_cell_new_column() {
    let mut book = Book::default();
    book.update_cell(
        &Address {
            sheet: 0,
            row: 1,
            col: 2,
        },
        "1",
    )
    .expect("failed to edit cell");
    let WorksheetDimension {
        min_row,
        max_row,
        min_column,
        max_column,
    } = book.get_dimensions().expect("couldn't get dimensions");
    assert_eq!((1, 1, 1, 2), (min_row, max_row, min_column, max_column));
    assert_eq!((1, 2), book.get_size().expect("Failed to get size"));
}

#[test]
fn test_book_insert_rows() {
    let mut book = Book::default();
    book.update_cell(
        &Address {
            sheet: 0,
            row: 2,
            col: 2,
        },
        "1",
    )
    .expect("failed to edit cell");
    book.move_to(&Address {
        sheet: 0,
        row: 2,
        col: 2,
    })
    .expect("Failed to move to location");
    assert_eq!((2, 2), book.get_size().expect("Failed to get size"));
    book.insert_rows(1, 5).expect("Failed to insert rows");
    assert_eq!((7, 2), book.get_size().expect("Failed to get size"));
    assert_eq!(
        Address {
            sheet: 0,
            row: 7,
            col: 2
        },
        book.location
    );
    assert_eq!(
        "1",
        book.get_current_cell_rendered()
            .expect("Failed to get rendered content")
    );
}

#[test]
fn test_book_insert_columns() {
    let mut book = Book::default();
    book.update_cell(
        &Address {
            sheet: 0,
            row: 2,
            col: 2,
        },
        "1",
    )
    .expect("failed to edit cell");
    book.move_to(&Address {
        sheet: 0,
        row: 2,
        col: 2,
    })
    .expect("Failed to move to location");
    assert_eq!((2, 2), book.get_size().expect("Failed to get size"));
    book.insert_columns(1, 5).expect("Failed to insert rows");
    assert_eq!((2, 7), book.get_size().expect("Failed to get size"));
    assert_eq!(
        Address {
            sheet: 0,
            row: 2,
            col: 7
        },
        book.location
    );
    assert_eq!(
        "1",
        book.get_current_cell_rendered()
            .expect("Failed to get rendered content")
    );
}

#[test]
fn test_book_col_size() {
    let mut book = Book::default();
    book.update_cell(
        &Address {
            sheet: 0,
            row: 2,
            col: 2,
        },
        "1",
    )
    .expect("failed to edit cell");
    book.set_col_size(1, 20).expect("Failed to set column size");
    assert_eq!(20, book.get_col_size(1).expect("Failed to get column size"));
}

#[test]
fn test_book_get_exportable_rows() {
    let mut book = Book::default();
    book.update_cell(
        &Address {
            sheet: 0,
            row: 1,
            col: 3,
        },
        "1-3",
    )
    .expect("failed to edit cell");
    book.update_cell(
        &Address {
            sheet: 0,
            row: 3,
            col: 6,
        },
        "3-6",
    )
    .expect("failed to edit cell");

    let rows = book.get_export_rows().expect("Failed to get export rows");
    assert_eq!(4, rows.len());
    assert_eq!(
        rows,
        vec![
            vec!["", "", "", "", "", "", ""],
            vec!["", "", "", "1-3", "", "", ""],
            vec!["", "", "", "", "", "", ""],
            vec!["", "", "", "", "", "", "3-6"],
        ]
    );
}
