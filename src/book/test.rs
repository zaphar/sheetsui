use ironcalc::base::worksheet::WorksheetDimension;

use crate::ui::Address;

use super::{Book, FileFormat};

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

#[test]
fn test_sheet_to_clipboard_content() {
    let mut book = Book::default();
    book.update_cell(
        &Address {
            sheet: 0,
            row: 1,
            col: 1,
        },
        "A1",
    )
    .expect("failed to edit cell");
    book.update_cell(
        &Address {
            sheet: 0,
            row: 1,
            col: 2,
        },
        "B1",
    )
    .expect("failed to edit cell");
    book.update_cell(
        &Address {
            sheet: 0,
            row: 2,
            col: 1,
        },
        "A2",
    )
    .expect("failed to edit cell");
    book.update_cell(
        &Address {
            sheet: 0,
            row: 2,
            col: 2,
        },
        "B2",
    )
    .expect("failed to edit cell");
    
    let (html, csv) = dbg!(book.sheeet_to_clipboard_content(0).expect("Failed to get clipboard content"));
    
    // Check that HTML contains table elements and our data
    assert!(html.contains("<table>"));
    assert!(html.contains("<tr>"));
    assert!(html.contains("<td>"));
    assert!(html.contains("A1"));
    assert!(html.contains("B1"));
    assert!(html.contains("A2"));
    assert!(html.contains("B2"));
    
    // Check CSV content
    let expected_csv = ",,\n,A1,B1\n,A2,B2\n";
    assert_eq!(csv, expected_csv);
}

#[test]
fn test_range_to_clipboard_content() {
    let mut book = Book::default();
    book.update_cell(
        &Address {
            sheet: 0,
            row: 1,
            col: 1,
        },
        "A1",
    )
    .expect("failed to edit cell");
    book.update_cell(
        &Address {
            sheet: 0,
            row: 1,
            col: 2,
        },
        "B1",
    )
    .expect("failed to edit cell");
    book.update_cell(
        &Address {
            sheet: 0,
            row: 2,
            col: 1,
        },
        "A2",
    )
    .expect("failed to edit cell");
    book.update_cell(
        &Address {
            sheet: 0,
            row: 2,
            col: 2,
        },
        "B2",
    )
    .expect("failed to edit cell");
    
    let start = Address { sheet: 0, row: 1, col: 1 };
    let end = Address { sheet: 0, row: 2, col: 2 };
    let range = super::AddressRange { start: &start, end: &end };
    
    let (html, csv) = book.range_to_clipboard_content(range).expect("Failed to get clipboard content");
    
    // Check that HTML contains table elements and our data
    assert!(html.contains("<table>"));
    assert!(html.contains("<tr>"));
    assert!(html.contains("<td>"));
    assert!(html.contains("A1"));
    assert!(html.contains("B1"));
    assert!(html.contains("A2"));
    assert!(html.contains("B2"));
    
    // Check CSV content
    let expected_csv = "A1,B1\nA2,B2\n";
    assert_eq!(csv, expected_csv);
}

// -------------------------------------------------------------------------
// Phase 2: Book I/O Integration and Format Detection (REQ-005..REQ-008)
// -------------------------------------------------------------------------

fn phase2_addr(row: usize, col: usize) -> Address {
    Address { sheet: 0, row, col }
}

fn tmp_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("sheetui_test_{name}"))
}

#[test]
fn test_new_book_default_format_is_sui() {
    // REQ-008: new/empty books default to SUI format.
    let book = Book::default();
    assert_eq!(
        book.get_format(),
        &FileFormat::Sui,
        "a new Book should default to FileFormat::Sui"
    );
    assert!(
        book.file_path.is_none(),
        "a new Book should have no file_path"
    );
}

#[test]
fn test_load_sui_file_by_extension() {
    // REQ-005: .sui extension → SUI parser.
    let path = tmp_path("load_sui.sui");
    let content = "[sheet \"TestSheet\"]\nA1 = \"loaded-from-sui\"\n[/sheet]\n";
    std::fs::write(&path, content).expect("write temp .sui");
    let book = Book::load(&path, "en", "America/New_York").expect("load .sui file");
    assert_eq!(
        book.get_format(),
        &FileFormat::Sui,
        "loading a .sui file should set format=Sui"
    );
    let val = book
        .get_cell_addr_contents(&phase2_addr(1, 1))
        .expect("get cell A1");
    assert_eq!(val, "loaded-from-sui");
    std::fs::remove_file(&path).ok();
}

#[test]
fn test_load_xlsx_file_by_extension() {
    // REQ-005: .xlsx extension → xlsx parser, format=Xlsx.
    let path = tmp_path("load_xlsx.xlsx");
    // Use the existing new_from_xlsx-style save path to create a real .xlsx
    let mut book = Book::default();
    book.update_cell(&phase2_addr(1, 1), "xlsx-load-test")
        .expect("set A1");
    book.evaluate();
    book.save_to_xlsx(&path.to_string_lossy()).expect("save_to_xlsx");
    // Now load via the new format-detecting Book::load
    let loaded = Book::load(&path, "en", "America/New_York").expect("load .xlsx");
    assert_eq!(
        loaded.get_format(),
        &FileFormat::Xlsx,
        "loading a .xlsx file should set format=Xlsx"
    );
    let val = loaded
        .get_cell_addr_contents(&phase2_addr(1, 1))
        .expect("get A1 after load");
    assert_eq!(val, "xlsx-load-test");
    std::fs::remove_file(&path).ok();
}

#[test]
fn test_save_as_sui_round_trip() {
    // REQ-006 / REQ-007: save_as writes the file; reloading it returns the same content.
    let path = tmp_path("save_as_sui.sui");
    let mut book = Book::default();
    book.update_cell(&phase2_addr(1, 1), "save-round-trip")
        .expect("set cell");
    book.evaluate();
    book.save_as(&path).expect("save_as .sui");
    let book2 = Book::load(&path, "en", "America/New_York").expect("reload .sui");
    let val = book2
        .get_cell_addr_contents(&phase2_addr(1, 1))
        .expect("get cell after reload");
    assert_eq!(val, "save-round-trip", "cell must survive save_as/load round-trip");
    std::fs::remove_file(&path).ok();
}

#[test]
fn test_save_updates_file_in_place() {
    // REQ-006: Book::save() overwrites the current file_path.
    let path = tmp_path("save_in_place.sui");
    let mut book = Book::default();
    book.update_cell(&phase2_addr(1, 1), "initial")
        .expect("set cell");
    book.evaluate();
    book.save_as(&path).expect("initial save_as");
    // Update and save in-place
    book.update_cell(&phase2_addr(1, 1), "updated")
        .expect("update cell");
    book.evaluate();
    book.save().expect("save in-place");
    let book2 = Book::load(&path, "en", "America/New_York").expect("reload");
    let val = book2
        .get_cell_addr_contents(&phase2_addr(1, 1))
        .expect("get cell");
    assert_eq!(val, "updated", "save() must overwrite the file in place");
    std::fs::remove_file(&path).ok();
}

#[test]
fn test_save_as_updates_stored_format_to_xlsx() {
    // REQ-007: save_as with a .xlsx path updates format to Xlsx.
    let path = tmp_path("format_change.xlsx");
    let mut book = Book::default();
    book.update_cell(&phase2_addr(1, 1), "format-change")
        .expect("set cell");
    book.evaluate();
    assert_eq!(book.get_format(), &FileFormat::Sui);
    book.save_as(&path).expect("save_as .xlsx");
    assert_eq!(
        book.get_format(),
        &FileFormat::Xlsx,
        "save_as .xlsx must update stored format to Xlsx"
    );
    std::fs::remove_file(&path).ok();
}

#[test]
fn test_save_as_subsequent_save_uses_new_format() {
    // REQ-007: after save_as changes the format, a bare save() uses the new format.
    let path_sui = tmp_path("subsequent_sui.sui");
    let path_xlsx = tmp_path("subsequent_xlsx.xlsx");
    let mut book = Book::default();
    book.update_cell(&phase2_addr(1, 1), "step1").expect("set");
    book.evaluate();
    book.save_as(&path_sui).expect("save as .sui");
    book.save_as(&path_xlsx).expect("save as .xlsx — changes format");
    book.update_cell(&phase2_addr(1, 1), "step2").expect("update");
    book.evaluate();
    book.save().expect("save with updated format");
    // The save should have written the xlsx file
    let book2 =
        Book::load(&path_xlsx, "en", "America/New_York").expect("reload .xlsx");
    let val = book2
        .get_cell_addr_contents(&phase2_addr(1, 1))
        .expect("get cell");
    assert_eq!(
        val, "step2",
        "save() must use the format last set by save_as"
    );
    std::fs::remove_file(&path_sui).ok();
    std::fs::remove_file(&path_xlsx).ok();
}

#[test]
fn test_save_without_path_returns_err() {
    // REQ-006: save() with no file_path must return Err, not panic.
    let mut book = Book::default();
    book.update_cell(&phase2_addr(1, 1), "no-path")
        .expect("set cell");
    let result = book.save();
    assert!(
        result.is_err(),
        "save() with no file_path must return Err, got: Ok"
    );
}

#[test]
fn test_cross_format_conversion_xlsx_to_sui() {
    // REQ-010 Phase 2: open xlsx (via save_to_xlsx), convert to .sui, reload, compare cells.
    let path_xlsx = tmp_path("cross_format.xlsx");
    let path_sui = tmp_path("cross_format.sui");
    // Build a book and save as xlsx using the existing save_to_xlsx method
    let mut book = Book::default();
    book.update_cell(&phase2_addr(1, 1), "cell-a1").expect("A1");
    book.update_cell(&phase2_addr(1, 2), "42").expect("B1");
    book.update_cell(&phase2_addr(2, 1), "=SUM(A1:B1)").expect("A2 formula");
    book.evaluate();
    book.save_to_xlsx(&path_xlsx.to_string_lossy()).expect("save xlsx");
    // Load the xlsx via format-detecting Book::load, then convert to sui
    let mut book2 =
        Book::load(&path_xlsx, "en", "America/New_York").expect("load .xlsx");
    book2.save_as(&path_sui).expect("save_as .sui");
    // Reload the .sui and verify cells survived
    let book3 =
        Book::load(&path_sui, "en", "America/New_York").expect("load .sui");
    let a1 = book3.get_cell_addr_contents(&phase2_addr(1, 1)).expect("A1");
    let b1 = book3.get_cell_addr_contents(&phase2_addr(1, 2)).expect("B1");
    let a2 = book3.get_cell_addr_contents(&phase2_addr(2, 1)).expect("A2");
    assert_eq!(a1, "cell-a1", "A1 must survive xlsx→sui conversion");
    assert_eq!(b1, "42", "B1 must survive xlsx→sui conversion");
    assert!(
        a2.starts_with('=') && a2.contains("SUM"),
        "formula in A2 must survive xlsx→sui conversion, got: {a2}"
    );
    std::fs::remove_file(&path_xlsx).ok();
    std::fs::remove_file(&path_sui).ok();
}
