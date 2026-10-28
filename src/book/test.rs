use ironcalc::base::{expressions::types::Area, worksheet::WorksheetDimension, Model};

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
    assert_eq!(3, rows.len());
    assert_eq!(
        rows,
        vec![
            vec!["", "", "1-3", "", "", ""],
            vec!["", "", "", "", "", ""],
            vec!["", "", "", "", "", "3-6"],
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
    let expected_csv = "A1,B1\nA2,B2\n";
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

// -------------------------------------------------------------------------
// CSV load and save
// -------------------------------------------------------------------------

fn csv_book(text: &str) -> Book {
    Book::from_csv_reader(text.as_bytes(), None).expect("parse csv")
}

fn rendered(book: &Book, row: usize, col: usize) -> String {
    book.get_cell_addr_rendered(&phase2_addr(row, col))
        .expect("render cell")
}

#[test]
fn test_csv_loads_rows_and_columns_from_a1() {
    let book = csv_book("a,b\nc,d\n");
    assert_eq!(rendered(&book, 1, 1), "a");
    assert_eq!(rendered(&book, 1, 2), "b");
    assert_eq!(rendered(&book, 2, 1), "c");
    assert_eq!(rendered(&book, 2, 2), "d");
    assert_eq!(book.get_format(), &FileFormat::Csv);
    assert!(!book.dirty, "a freshly loaded book is not dirty");
}

#[test]
fn test_csv_keeps_first_row_as_data() {
    let book = csv_book("name,qty\nfoo,1\n");
    assert_eq!(rendered(&book, 1, 1), "name");
    assert_eq!(rendered(&book, 2, 1), "foo");
}

#[test]
fn test_csv_accepts_ragged_rows() {
    let book = csv_book("a\nb,c,d\ne,f\n");
    assert_eq!(rendered(&book, 1, 1), "a");
    assert_eq!(rendered(&book, 2, 3), "d");
    assert_eq!(rendered(&book, 3, 2), "f");
}

#[test]
fn test_csv_numbers_are_numbers() {
    let mut book = csv_book("1,-2.5,+3\n");
    book.update_cell(&phase2_addr(2, 1), "=A1+B1+C1").expect("set formula");
    book.evaluate();
    assert_eq!(rendered(&book, 2, 1), "1.5");
}

#[test]
fn test_csv_formula_text_stays_literal() {
    let book = csv_book("=1+1,-A1,+B2,'quoted\n");
    assert_eq!(rendered(&book, 1, 1), "=1+1");
    assert_eq!(rendered(&book, 1, 2), "-A1");
    assert_eq!(rendered(&book, 1, 3), "+B2");
    assert_eq!(rendered(&book, 1, 4), "'quoted");
}

#[test]
fn test_csv_quoted_fields() {
    let book = csv_book("\"a,b\",\"line1\nline2\",\"say \"\"hi\"\"\"\n");
    assert_eq!(rendered(&book, 1, 1), "a,b");
    assert_eq!(rendered(&book, 1, 2), "line1\nline2");
    assert_eq!(rendered(&book, 1, 3), "say \"hi\"");
}

#[test]
fn test_csv_empty_input() {
    let book = csv_book("");
    assert_eq!(book.get_export_rows_for_sheet(0).expect("rows").len(), 0);
}

#[test]
fn test_load_csv_file_by_extension_names_sheet_from_stem() {
    let path = tmp_path("load_csv.CSV");
    std::fs::write(&path, "x,1\n").expect("write temp .csv");
    let book = Book::load(&path, "en", "America/New_York").expect("load .csv");
    assert_eq!(book.get_format(), &FileFormat::Csv);
    assert_eq!(book.get_sheet_names(), vec!["sheetui_test_load_csv".to_string()]);
    assert_eq!(rendered(&book, 1, 1), "x");
    std::fs::remove_file(&path).ok();
}

#[test]
fn test_csv_save_round_trip_is_stable() {
    let path = tmp_path("round_trip.csv");
    let content = "a,1\n=x,\"q,r\"\n";
    std::fs::write(&path, content).expect("write temp .csv");
    let mut book = Book::load(&path, "en", "America/New_York").expect("load .csv");
    book.save().expect("save .csv");
    assert_eq!(std::fs::read_to_string(&path).expect("read back"), content);
    std::fs::remove_file(&path).ok();
}

#[test]
fn test_csv_save_writes_first_sheet_values() {
    let path = tmp_path("save_values.csv");
    let mut book = Book::default();
    book.update_cell(&phase2_addr(1, 1), "2").expect("set A1");
    book.update_cell(&phase2_addr(1, 2), "=A1*3").expect("set B1");
    book.new_sheet(Some("Other")).expect("add sheet");
    book.update_cell(&Address { sheet: 1, row: 1, col: 1 }, "ignored")
        .expect("set Other!A1");
    book.evaluate();
    book.save_as(&path).expect("save_as .csv");
    assert_eq!(book.get_format(), &FileFormat::Csv);
    assert_eq!(std::fs::read_to_string(&path).expect("read back"), "2,6\n");
    std::fs::remove_file(&path).ok();
}

// -------------------------------------------------------------------------
// Sort rows
// -------------------------------------------------------------------------

fn key(col: usize, ascending: bool) -> super::sort::SortKey {
    super::sort::SortKey { col, ascending }
}

fn column(book: &Book, col: usize, rows: std::ops::RangeInclusive<usize>) -> Vec<String> {
    rows.map(|r| rendered(book, r, col)).collect()
}

#[test]
fn test_sort_rows_ascending_moves_whole_rows() {
    let mut book = csv_book("3,c\n1,a\n2,b\n");
    book.sort_rows(0, 1, 3, &[key(1, true)]).expect("sort");
    assert_eq!(column(&book, 1, 1..=3), vec!["1", "2", "3"]);
    assert_eq!(column(&book, 2, 1..=3), vec!["a", "b", "c"]);
    assert!(book.dirty);
}

#[test]
fn test_sort_rows_descending() {
    let mut book = csv_book("1\n3\n2\n");
    book.sort_rows(0, 1, 3, &[key(1, false)]).expect("sort");
    assert_eq!(column(&book, 1, 1..=3), vec!["3", "2", "1"]);
}

#[test]
fn test_sort_rows_only_touches_given_rows() {
    let mut book = csv_book("h\n3\n1\n2\nf\n");
    book.sort_rows(0, 2, 4, &[key(1, true)]).expect("sort");
    assert_eq!(column(&book, 1, 1..=5), vec!["h", "1", "2", "3", "f"]);
}

#[test]
fn test_sort_rows_multiple_keys() {
    let mut book = csv_book("1,a\n2,a\n1,b\n");
    book.sort_rows(0, 1, 3, &[key(1, true), key(2, false)])
        .expect("sort");
    assert_eq!(column(&book, 1, 1..=3), vec!["1", "1", "2"]);
    assert_eq!(column(&book, 2, 1..=3), vec!["b", "a", "a"]);
}

#[test]
fn test_sort_rows_is_stable() {
    let mut book = csv_book("1,x\n0,y\n1,z\n");
    book.sort_rows(0, 1, 3, &[key(1, true)]).expect("sort");
    assert_eq!(column(&book, 2, 1..=3), vec!["y", "x", "z"]);
}

#[test]
fn test_sort_rows_blanks_last_in_both_directions() {
    let mut book = csv_book("2,a\n,b\n1,c\n");
    book.sort_rows(0, 1, 3, &[key(1, true)]).expect("sort asc");
    assert_eq!(column(&book, 2, 1..=3), vec!["c", "a", "b"]);
    book.sort_rows(0, 1, 3, &[key(1, false)]).expect("sort desc");
    assert_eq!(column(&book, 2, 1..=3), vec!["a", "c", "b"]);
}

#[test]
fn test_sort_rows_mixed_types_use_excel_order() {
    let mut book = Book::default();
    for (row, value) in ["=1/0", "TRUE", "text", "5"].iter().enumerate() {
        book.update_cell(&phase2_addr(row + 1, 1), *value).expect("set");
    }
    book.evaluate();
    book.sort_rows(0, 1, 4, &[key(1, true)]).expect("sort asc");
    assert_eq!(column(&book, 1, 1..=4), vec!["5", "text", "TRUE", "#DIV/0!"]);
    book.sort_rows(0, 1, 4, &[key(1, false)]).expect("sort desc");
    assert_eq!(column(&book, 1, 1..=4), vec!["#DIV/0!", "TRUE", "text", "5"]);
}

#[test]
fn test_sort_rows_text_ignores_case() {
    let mut book = csv_book("b\nA\nc\n");
    book.sort_rows(0, 1, 3, &[key(1, true)]).expect("sort");
    assert_eq!(column(&book, 1, 1..=3), vec!["A", "b", "c"]);
}

#[test]
fn test_sort_rows_formulas_sort_by_value_and_translate() {
    let mut book = csv_book("3\n1\n2\n");
    for row in 1..=3 {
        book.update_cell(&phase2_addr(row, 2), format!("=A{row}*-1"))
            .expect("set formula");
    }
    book.evaluate();
    // Column B is -A, so ascending B is descending A.
    book.sort_rows(0, 1, 3, &[key(2, true)]).expect("sort");
    assert_eq!(column(&book, 1, 1..=3), vec!["3", "2", "1"]);
    assert_eq!(column(&book, 2, 1..=3), vec!["-3", "-2", "-1"]);
    for row in 1..=3 {
        assert_eq!(
            book.get_cell_addr_contents(&phase2_addr(row, 2)).expect("contents"),
            format!("=A{row}*-1")
        );
    }
}

#[test]
fn test_sort_rows_keeps_literal_text_as_text() {
    let mut book = csv_book("2,=1+1\n1,-A1\n");
    book.sort_rows(0, 1, 2, &[key(1, true)]).expect("sort");
    assert_eq!(column(&book, 2, 1..=2), vec!["-A1", "=1+1"]);
}

#[test]
fn test_sort_rows_moves_cell_styles() {
    let mut book = csv_book("2\n1\n");
    let area = Area { sheet: 0, row: 1, column: 1, width: 1, height: 1 };
    book.set_cell_style(&[("font.b", "true")], &area).expect("bold");
    book.sort_rows(0, 1, 2, &[key(1, true)]).expect("sort");
    let bold = |row| book.get_cell_style(&phase2_addr(row, 1)).map(|s| s.font.b);
    assert_eq!(bold(1), Some(false));
    assert_eq!(bold(2), Some(true));
}

#[test]
fn test_sort_rows_does_not_move_row_height() {
    let mut book = csv_book("2\n1\n");
    book.model.set_rows_height(0, 1, 1, 40.0).expect("set height");
    let default_height = book.model.get_row_height(0, 2).expect("height");
    book.sort_rows(0, 1, 2, &[key(1, true)]).expect("sort");
    assert_eq!(book.model.get_row_height(0, 1).expect("height"), 40.0);
    assert_eq!(book.model.get_row_height(0, 2).expect("height"), default_height);
}

#[test]
fn test_sort_rows_already_sorted_is_not_a_change() {
    let mut book = csv_book("1\n2\n");
    book.sort_rows(0, 1, 2, &[key(1, true)]).expect("sort");
    assert!(!book.dirty);
}

#[test]
fn test_sort_rows_duplicate_key_is_an_error() {
    let mut book = csv_book("2,a\n1,b\n");
    let err = book
        .sort_rows(0, 1, 2, &[key(2, true), key(1, true), key(2, false)])
        .expect_err("duplicate key");
    assert_eq!(err.to_string(), "Column B is given more than one time.");
    assert_eq!(column(&book, 1, 1..=2), vec!["2", "1"]);
}

#[test]
fn test_sort_rows_merged_cells_are_an_error() {
    let mut model =
        Model::new_empty("test", "en", "America/New_York", "en").expect("model");
    model.workbook.worksheets[0].merge_cells.push("A2:B3".to_string());
    let mut book = Book::from_model(model);
    for (row, value) in ["3", "1", "2"].iter().enumerate() {
        book.update_cell(&phase2_addr(row + 1, 1), *value).expect("set");
    }
    book.evaluate();
    let err = book.sort_rows(0, 1, 3, &[key(1, true)]).expect_err("merged");
    assert_eq!(err.to_string(), "Cannot sort a range that contains merged cells.");
    assert_eq!(column(&book, 1, 1..=3), vec!["3", "1", "2"]);
}

#[test]
fn test_sort_rows_array_formulas_are_an_error() {
    let mut book = csv_book("3,\n1,\n2,\n");
    book.update_cell(&phase2_addr(1, 2), "=SORT(A1:A3)").expect("set");
    book.evaluate();
    let err = book.sort_rows(0, 1, 3, &[key(1, true)]).expect_err("array");
    assert_eq!(err.to_string(), "Cannot sort a range that contains array formulas.");
    assert_eq!(column(&book, 1, 1..=3), vec!["3", "1", "2"]);
}

// -------------------------------------------------------------------------
// Undo and redo
// -------------------------------------------------------------------------

/// Everything a change can touch in the first `rows` × `cols` cells: sheet
/// names, column widths, and each cell's contents and style.
fn snapshot(book: &Book, rows: usize, cols: usize) -> String {
    let mut out = format!("{:?}\n", book.get_sheet_names());
    for sheet in 0..book.get_sheet_names().len() as u32 {
        for col in 1..=cols {
            let width = book.get_column_size_for_sheet(sheet, col).expect("width");
            out.push_str(&format!("{sheet} width {col}: {width}\n"));
        }
        for row in 1..=rows {
            for col in 1..=cols {
                let addr = Address { sheet, row, col };
                let contents = book.get_cell_addr_contents(&addr).expect("contents");
                let style = book.get_cell_style(&addr);
                out.push_str(&format!("{sheet} {row},{col}: {contents:?} {style:?}\n"));
            }
        }
    }
    out
}

type Change = fn(&mut Book) -> anyhow::Result<()>;

#[test]
fn test_undo_and_redo_each_change() {
    let changes: Vec<(&str, Change)> = vec![
        ("update_cell", |b| b.update_cell(&phase2_addr(1, 1), "x")),
        ("formula", |b| b.update_cell(&phase2_addr(1, 3), "=A2*2")),
        ("clear_cell_contents", |b| b.clear_cell_contents(phase2_addr(1, 1))),
        ("clear_cell_all", |b| b.clear_cell_all(phase2_addr(2, 2))),
        ("clear_cell_range", |b| {
            b.clear_cell_range(phase2_addr(1, 1), phase2_addr(2, 2))
        }),
        ("clear_cell_range_all", |b| {
            b.clear_cell_range_all(phase2_addr(1, 1), phase2_addr(2, 2))
        }),
        ("set_cell_style", |b| {
            let area = Area { sheet: 0, row: 1, column: 1, width: 2, height: 1 };
            b.set_cell_style(&[("font.b", "true"), ("fill.color", "#ff0000")], &area)
        }),
        ("extend_to", |b| b.extend_to(&phase2_addr(1, 1), &phase2_addr(3, 1))),
        ("insert_rows", |b| b.insert_rows(1, 2)),
        ("insert_columns", |b| b.insert_columns(1, 2)),
        ("sort_rows", |b| b.sort_rows(0, 1, 3, &[key(1, false)])),
        ("set_col_size", |b| b.set_col_size(1, 30)),
        ("set_sheet_name", |b| b.set_sheet_name(0, "Renamed")),
        ("new_sheet", |b| b.new_sheet(Some("Other"))),
    ];
    for (name, change) in changes {
        let mut book = csv_book("1,2\n3,4\n5,6\n");
        book.set_cell_style(
            &[("font.i", "true")],
            &Area { sheet: 0, row: 2, column: 2, width: 1, height: 1 },
        )
        .expect("style");
        book.evaluate();
        let before = snapshot(&book, 5, 4);
        change(&mut book).expect(name);
        book.evaluate();
        let after = snapshot(&book, 5, 4);
        assert_ne!(before, after, "{name} changes the book");

        assert!(book.undo().expect("undo"), "{name} can be undone");
        assert_eq!(snapshot(&book, 5, 4), before, "undo of {name}");
        assert!(book.redo().expect("redo"), "{name} can be redone");
        assert_eq!(snapshot(&book, 5, 4), after, "redo of {name}");
        assert!(book.undo().expect("undo"));
        assert!(book.undo().expect("undo"), "the style is the action before {name}");
        assert!(!book.undo().expect("undo"), "{name} is one action");
    }
}

#[test]
fn test_undo_of_a_sort_restores_formulas_and_styles() {
    let mut book = csv_book("3,c\n1,a\n2,b\n");
    book.update_cell(&phase2_addr(1, 3), "=A1*2").expect("formula");
    book.set_cell_style(
        &[("font.b", "true")],
        &Area { sheet: 0, row: 1, column: 2, width: 1, height: 1 },
    )
    .expect("style");
    book.evaluate();
    let before = snapshot(&book, 3, 3);
    book.sort_rows(0, 1, 3, &[key(1, true)]).expect("sort");
    assert_eq!(column(&book, 1, 1..=3), vec!["1", "2", "3"]);
    assert!(book.undo().expect("undo"));
    assert_eq!(snapshot(&book, 3, 3), before);
    assert_eq!(column(&book, 3, 1..=3), vec!["6", "", ""]);
}

#[test]
fn test_a_loaded_book_has_nothing_to_undo() {
    let mut book = csv_book("1,2\n3,4\n");
    assert!(!book.undo().expect("undo"));
    assert!(!book.redo().expect("redo"));
    let mut book = Book::default();
    assert!(!book.undo().expect("undo"));
}

#[test]
fn test_a_sort_that_changes_nothing_is_not_an_action() {
    let mut book = csv_book("1\n2\n3\n");
    book.sort_rows(0, 1, 3, &[key(1, true)]).expect("sort");
    assert!(!book.undo().expect("undo"));
}

#[test]
fn test_changes_inside_an_action_undo_together() {
    let mut book = csv_book("1,2\n");
    book.begin_action();
    book.update_cell(&phase2_addr(1, 1), "a").expect("set");
    book.begin_action();
    book.update_cell(&phase2_addr(1, 2), "b").expect("set");
    book.end_action();
    book.update_cell(&phase2_addr(2, 1), "c").expect("set");
    book.end_action();
    book.evaluate();
    assert!(book.undo().expect("undo"));
    assert_eq!(column(&book, 1, 1..=2), vec!["1", ""]);
    assert_eq!(rendered(&book, 1, 2), "2");
    assert!(!book.undo().expect("undo"));
}

#[test]
fn test_an_action_with_no_change_is_not_kept() {
    let mut book = csv_book("1\n");
    book.update_cell(&phase2_addr(1, 1), "2").expect("set");
    book.begin_action();
    book.end_action();
    assert!(book.undo().expect("undo"));
    assert_eq!(rendered(&book, 1, 1), "1");
}

#[test]
fn test_undo_and_redo_go_to_the_change() {
    let mut book = csv_book("1,2\n");
    book.new_sheet(Some("Other")).expect("new sheet");
    assert!(book.select_sheet_by_name("Other"));
    book.move_to(&Address { sheet: 1, row: 4, col: 3 }).expect("move");
    book.edit_current_cell("x").expect("edit");
    assert!(book.select_sheet_by_name("Sheet1"));
    book.move_to(&Address { sheet: 0, row: 1, col: 1 }).expect("move");

    assert!(book.undo().expect("undo"));
    assert_eq!(book.location, Address { sheet: 1, row: 4, col: 3 });
    assert_eq!(book.model.get_selected_view().sheet, 1);
    book.move_to(&Address { sheet: 1, row: 1, col: 1 }).expect("move");
    assert!(book.redo().expect("redo"));
    assert_eq!(book.location, Address { sheet: 1, row: 4, col: 3 });
}

#[test]
fn test_undo_of_new_sheet_removes_it() {
    let mut book = csv_book("1\n");
    book.new_sheet(Some("Other")).expect("new sheet");
    assert_eq!(book.get_sheet_names().len(), 2);
    assert!(book.undo().expect("undo"));
    assert_eq!(book.get_sheet_names().len(), 1);
    assert_eq!(book.location.sheet, 0);
}

#[test]
fn test_a_new_change_clears_redo() {
    let mut book = csv_book("1,2\n");
    book.update_cell(&phase2_addr(1, 1), "a").expect("set");
    assert!(book.undo().expect("undo"));
    book.update_cell(&phase2_addr(1, 2), "b").expect("set");
    assert!(!book.redo().expect("redo"));
}

#[test]
fn test_undo_back_to_the_save_point_is_not_a_change() {
    let path = tmp_path("undo_save_point.sui");
    let mut book = csv_book("1,2\n");
    book.update_cell(&phase2_addr(1, 1), "a").expect("set");
    assert!(book.undo().expect("undo"));
    assert!(!book.dirty, "back at the loaded state");

    book.update_cell(&phase2_addr(1, 1), "a").expect("set");
    book.save_as(&path).expect("save");
    assert!(!book.dirty);
    book.update_cell(&phase2_addr(1, 2), "b").expect("set");
    assert!(book.dirty);
    assert!(book.undo().expect("undo"));
    assert!(!book.dirty, "back at the saved state");
    assert!(book.redo().expect("redo"));
    assert!(book.dirty);

    // A new change below the save point: the save point is gone.
    assert!(book.undo().expect("undo"));
    assert!(book.undo().expect("undo"));
    assert!(book.dirty);
    book.update_cell(&phase2_addr(2, 1), "c").expect("set");
    assert!(book.undo().expect("undo"));
    assert!(book.dirty, "the saved state can no longer be reached");
    std::fs::remove_file(&path).expect("clean up");
}
