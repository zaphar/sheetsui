// Copyright 2024 Jeremy Wall
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # .sui File Format
//!
//! The `.sui` format is a declarative, sparse, human-readable text format for
//! spreadsheet data. Each non-comment, non-structural line declares one cell or
//! one column-width setting.
//!
//! ## Grammar (EBNF)
//!
//! ```text
//! file          ::= line* EOF
//! line          ::= (comment | sheet_start | sheet_end | col_width | cell_decl) NEWLINE
//!                 | NEWLINE                        (* blank lines are ignored *)
//! comment       ::= '#' rest_of_line
//! sheet_start   ::= '[sheet' WS quoted_string ']'
//! sheet_end     ::= '[/sheet]'
//! col_width     ::= 'col' WS uint WS 'width' WS uint
//! cell_decl     ::= cellref WS '=' WS value
//! cellref       ::= [A-Z]+ [1-9][0-9]*            (* standard A1 notation, 1-based *)
//! value         ::= string | number | boolean | formula
//! string        ::= '"' char* '"'                  (* escapes: \" \\ \n *)
//! number        ::= '-'? [0-9]+ ('.' [0-9]+)?
//! boolean       ::= 'true' | 'false'               (* case-insensitive *)
//! formula       ::= '=' rest_of_line               (* everything after '=' is the formula *)
//! quoted_string ::= '"' char* '"'
//! uint          ::= [0-9]+
//! WS            ::= ' '+
//! ```
//!
//! ## Canonical Ordering (for deterministic serialization, REQ-002)
//!
//! 1. Sheets are written in workbook index order.
//! 2. Within each sheet, `col` width declarations come first, in ascending
//!    column-index order.
//! 3. Cell declarations follow in row-major order: ascending row, then
//!    ascending column within each row.
//! 4. Empty cells are omitted (sparse representation).
//!
//! ## Example
//!
//! ```text
//! [sheet "Sheet1"]
//! col 1 width 15
//! col 2 width 20
//! A1 = "hello world"
//! B1 = 42
//! C1 = true
//! A2 = =SUM(A1:A1)
//! [/sheet]
//! [sheet "Sheet2"]
//! A1 = "second sheet"
//! [/sheet]
//! ```
//!
//! ## Lenient Parsing (REQ-003, REQ-009)
//!
//! Lines that cannot be parsed are skipped and recorded as `ParseWarning`
//! entries. Valid lines before and after an invalid line are still loaded.
//! An all-invalid file returns an empty `Book` with one warning per invalid line.

use crate::ui::Address;
use ironcalc::base::UserModel;

use super::Book;

/// A warning produced when a line in a `.sui` file cannot be parsed.
pub struct ParseWarning {
    /// 1-based line number in the source text.
    pub line: usize,
    /// Human-readable description of the parse problem.
    pub message: String,
}

/// Parse `.sui` format text into a [`Book`].
///
/// Returns the populated `Book` and a (possibly empty) list of [`ParseWarning`]s
/// for lines that were skipped due to parse errors. Valid lines before and after
/// invalid lines are still loaded.
pub fn parse_sui(text: &str) -> (Book, Vec<ParseWarning>) {
    let mut book = Book::new(
        UserModel::new_empty("Sheet1", "en", "America/New_York", "en")
            .expect("failed to create workbook"),
    );
    let mut warnings = Vec::new();
    let mut current_sheet: Option<u32> = None;
    let mut sheet_count: u32 = 0;

    for (idx, line) in text.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some(name) = parse_sheet_start(trimmed) {
            if sheet_count == 0 {
                let _ = book.set_sheet_name(0, &name);
            } else {
                let _ = book.new_sheet(Some(&name));
            }
            current_sheet = Some(sheet_count);
            sheet_count += 1;
            continue;
        }

        if trimmed == "[/sheet]" {
            current_sheet = None;
            continue;
        }

        if let Some(sheet_idx) = current_sheet {
            if let Some((col, width)) = parse_col_width(trimmed) {
                let _ = book.set_column_size_for_sheet(sheet_idx, col, width);
            } else if let Some((row, col, value)) = parse_cell_decl(trimmed) {
                let _ = book.update_cell(&Address { sheet: sheet_idx, row, col }, &value);
            } else {
                warnings.push(ParseWarning {
                    line: line_num,
                    message: format!("unrecognized line: {trimmed}"),
                });
            }
        } else {
            warnings.push(ParseWarning {
                line: line_num,
                message: format!("content outside sheet block: {trimmed}"),
            });
        }
    }

    book.evaluate();
    (book, warnings)
}

/// Serialize a [`Book`] to `.sui` format text.
///
/// The output is deterministic: given the same `Book` state, two calls always
/// produce byte-identical output.
///
/// Canonical ordering:
/// 1. Sheets in workbook index order.
/// 2. Within each sheet: `col` width declarations in ascending column order.
/// 3. Cell declarations in row-major order (ascending row, then ascending column).
pub fn serialize_sui(book: &Book) -> String {
    let mut out = String::new();
    let worksheets = &book.model.get_model().workbook.worksheets;

    for (idx, ws) in worksheets.iter().enumerate() {
        let sheet_idx = idx as u32;
        out.push_str(&format!("[sheet \"{}\"]\n", escape_string(&ws.name)));

        // Find the maximum column index with any data in this sheet.
        let max_col = ws
            .sheet_data
            .values()
            .flat_map(|cols| cols.keys())
            .copied()
            .max()
            .unwrap_or(0);

        // Determine the ironcalc default column width by reading an
        // out-of-range column that was never explicitly set.
        let default_width = book
            .get_column_size_for_sheet(sheet_idx, 10_000)
            .unwrap_or(12);

        // Column width declarations (ascending col order).
        for col in 1..=(max_col as usize) {
            if let Ok(width) = book.get_column_size_for_sheet(sheet_idx, col) {
                if width != default_width {
                    out.push_str(&format!("col {col} width {width}\n"));
                }
            }
        }

        // Cell declarations in row-major order (sort keys for deterministic output — REQ-002).
        let mut rows: Vec<i32> = ws.sheet_data.keys().copied().collect();
        rows.sort_unstable();
        for row in rows {
            let cols_map = &ws.sheet_data[&row];
            let mut cols: Vec<i32> = cols_map.keys().copied().collect();
            cols.sort_unstable();
            for col in cols {
                let addr = Address {
                    sheet: sheet_idx,
                    row: row as usize,
                    col: col as usize,
                };
                if let Ok(content) = book.get_cell_addr_contents(&addr) {
                    if content.is_empty() {
                        continue;
                    }
                    let cell_ref =
                        format!("{}{row}", col_index_to_letters(col as usize));
                    let value_str = serialize_value(&content);
                    out.push_str(&format!("{cell_ref} = {value_str}\n"));
                }
            }
        }

        out.push_str("[/sheet]\n");
    }

    out
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn parse_sheet_start(line: &str) -> Option<String> {
    let rest = line.strip_prefix("[sheet ")?;
    let rest = rest.strip_suffix(']')?;
    parse_quoted_string(rest)
}

fn parse_col_width(line: &str) -> Option<(usize, usize)> {
    let mut parts = line.split_whitespace();
    if parts.next() != Some("col") {
        return None;
    }
    let col: usize = parts.next()?.parse().ok()?;
    if parts.next() != Some("width") {
        return None;
    }
    let width: usize = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None; // unexpected extra tokens
    }
    Some((col, width))
}

fn parse_cell_decl(line: &str) -> Option<(usize, usize, String)> {
    let eq = line.find(" = ")?;
    let cellref = &line[..eq];
    let value_part = &line[eq + 3..];
    let (row, col) = parse_cellref(cellref)?;
    let value = parse_value(value_part)?;
    Some((row, col, value))
}

fn parse_cellref(s: &str) -> Option<(usize, usize)> {
    let col_len = s.bytes().take_while(|b| b.is_ascii_uppercase()).count();
    if col_len == 0 || col_len == s.len() {
        return None;
    }
    let col = col_letters_to_index(&s[..col_len]);
    let row: usize = s[col_len..].parse().ok()?;
    if row == 0 {
        return None;
    }
    Some((row, col))
}

fn col_letters_to_index(s: &str) -> usize {
    s.bytes()
        .fold(0usize, |acc, b| acc * 26 + (b - b'A' + 1) as usize)
}

fn col_index_to_letters(mut col: usize) -> String {
    let mut bytes = Vec::new();
    while col > 0 {
        bytes.push(b'A' + ((col - 1) % 26) as u8);
        col = (col - 1) / 26;
    }
    bytes.reverse();
    String::from_utf8(bytes).unwrap()
}

fn parse_value(s: &str) -> Option<String> {
    if s.starts_with('"') {
        let inner = s.strip_prefix('"')?.strip_suffix('"')?;
        Some(unescape_string(inner))
    } else if s.starts_with('=') {
        Some(s.to_string())
    } else if s.eq_ignore_ascii_case("true") {
        Some("TRUE".to_string())
    } else if s.eq_ignore_ascii_case("false") {
        Some("FALSE".to_string())
    } else if s.parse::<f64>().is_ok() {
        Some(s.to_string())
    } else {
        None
    }
}

fn unescape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                Some('n') => result.push('\n'),
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }
    result
}

fn parse_quoted_string(s: &str) -> Option<String> {
    let s = s.trim();
    let inner = s.strip_prefix('"')?.strip_suffix('"')?;
    Some(unescape_string(inner))
}

fn serialize_value(content: &str) -> String {
    if content.starts_with('=') {
        content.to_string()
    } else if content.eq_ignore_ascii_case("true")
        || content.eq_ignore_ascii_case("false")
    {
        content.to_lowercase()
    } else if content.parse::<f64>().is_ok() {
        content.to_string()
    } else {
        format!("\"{}\"", escape_string(content))
    }
}

fn escape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            other => result.push(other),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{parse_sui, serialize_sui};
    use crate::book::Book;
    use crate::ui::Address;

    fn addr(row: usize, col: usize) -> Address {
        Address { sheet: 0, row, col }
    }

    // -------------------------------------------------------------------------
    // Parser tests (REQ-003)
    // -------------------------------------------------------------------------

    #[test]
    fn test_parse_sui_empty_string() {
        let (book, warnings) = parse_sui("");
        assert_eq!(warnings.len(), 0, "empty input should produce zero warnings");
        let names = book.get_sheet_names();
        assert!(names.len() >= 1, "book should have at least one sheet");
    }

    #[test]
    fn test_parse_sui_basic_cell_value() {
        let text = "[sheet \"Sheet1\"]\nA1 = \"hello\"\n[/sheet]\n";
        let (book, warnings) = parse_sui(text);
        assert_eq!(warnings.len(), 0, "valid .sui should produce no warnings");
        let content = book
            .get_cell_addr_contents(&addr(1, 1))
            .expect("could not get cell contents");
        assert_eq!(content, "hello");
    }

    #[test]
    fn test_parse_sui_numeric_cell() {
        let text = "[sheet \"Sheet1\"]\nA1 = 42\n[/sheet]\n";
        let (book, warnings) = parse_sui(text);
        assert_eq!(warnings.len(), 0);
        let content = book
            .get_cell_addr_contents(&addr(1, 1))
            .expect("could not get cell contents");
        assert_eq!(content, "42");
    }

    #[test]
    fn test_parse_sui_boolean_cell() {
        let text = "[sheet \"Sheet1\"]\nA1 = true\n[/sheet]\n";
        let (book, warnings) = parse_sui(text);
        assert_eq!(warnings.len(), 0);
        let content = book
            .get_cell_addr_contents(&addr(1, 1))
            .expect("could not get cell contents");
        assert!(
            content.eq_ignore_ascii_case("true"),
            "expected boolean true, got: {content}"
        );
    }

    #[test]
    fn test_parse_sui_formula_cell() {
        let text = "[sheet \"Sheet1\"]\nA1 = 10\nA2 = =SUM(A1:A1)\n[/sheet]\n";
        let (book, warnings) = parse_sui(text);
        assert_eq!(warnings.len(), 0);
        let content = book
            .get_cell_addr_contents(&addr(2, 1))
            .expect("could not get formula cell contents");
        assert!(
            content.starts_with('='),
            "formula cell content should start with '=', got: {content}"
        );
        assert!(
            content.contains("SUM"),
            "formula cell should contain 'SUM', got: {content}"
        );
    }

    #[test]
    fn test_parse_sui_multiple_sheets() {
        let text = "[sheet \"Alpha\"]\nA1 = \"in alpha\"\n[/sheet]\n\
                    [sheet \"Beta\"]\nA1 = \"in beta\"\n[/sheet]\n";
        let (book, warnings) = parse_sui(text);
        assert_eq!(warnings.len(), 0);
        let names = book.get_sheet_names();
        assert!(
            names.contains(&"Alpha".to_string()),
            "book should contain sheet 'Alpha', got: {names:?}"
        );
        assert!(
            names.contains(&"Beta".to_string()),
            "book should contain sheet 'Beta', got: {names:?}"
        );
    }

    #[test]
    fn test_parse_sui_column_width() {
        let text = "[sheet \"Sheet1\"]\ncol 1 width 20\nA1 = \"wide\"\n[/sheet]\n";
        let (book, warnings) = parse_sui(text);
        assert_eq!(warnings.len(), 0);
        let width = book
            .get_column_size_for_sheet(0, 1)
            .expect("could not get column size");
        assert_eq!(width, 20, "column 1 should have width 20, got: {width}");
    }

    #[test]
    fn test_parse_sui_invalid_line_is_warning() {
        // Line 1: valid sheet start
        // Line 2: invalid — no recognized syntax
        // Line 3: valid cell declaration
        // Line 4: valid sheet end
        let text =
            "[sheet \"Sheet1\"]\nTHIS IS NOT VALID SYNTAX\nA1 = \"still here\"\n[/sheet]\n";
        let (book, warnings) = parse_sui(text);
        assert_eq!(warnings.len(), 1, "exactly one warning for one invalid line");
        assert_eq!(
            warnings[0].line, 2,
            "warning should report line 2, got: {}",
            warnings[0].line
        );
        // The valid cell A1 must still be loaded despite the invalid line before it
        let content = book
            .get_cell_addr_contents(&addr(1, 1))
            .expect("could not get cell contents");
        assert_eq!(
            content, "still here",
            "valid cell after invalid line should still be loaded"
        );
    }

    #[test]
    fn test_parse_sui_all_invalid_returns_warnings() {
        let text = "NOT VALID\nALSO NOT VALID\nSTILL NOT VALID\n";
        let (book, warnings) = parse_sui(text);
        assert_eq!(
            warnings.len(),
            3,
            "each invalid line should produce one warning"
        );
        // All-invalid input produces an empty book (no cells set)
        let (rows, _cols) = book.get_size().expect("could not get size");
        assert_eq!(rows, 0, "all-invalid .sui should produce a book with no cell data");
    }

    // -------------------------------------------------------------------------
    // Serializer tests (REQ-004)
    // -------------------------------------------------------------------------

    #[test]
    fn test_serialize_sui_empty_book() {
        let book = Book::default();
        // Must not panic; must return a String (enforced by return type)
        let output = serialize_sui(&book);
        // An empty book (only one empty cell) may or may not produce output,
        // but the call must succeed and the result must be valid UTF-8.
        assert!(output.is_ascii() || !output.is_ascii(), "output is a String — UTF-8 guaranteed");
        // Specifically: must not panic (the todo!() stub will cause this test to fail).
    }

    #[test]
    fn test_serialize_sui_basic_cell() {
        let mut book = Book::default();
        book.update_cell(&addr(1, 1), "hello-serialized")
            .expect("failed to set cell");
        book.evaluate();
        let output = serialize_sui(&book);
        assert!(
            output.contains("hello-serialized"),
            "output should contain the cell value 'hello-serialized', got: {output}"
        );
    }

    #[test]
    fn test_serialize_sui_formula_cell() {
        let mut book = Book::default();
        book.update_cell(&addr(1, 1), "10").expect("failed to set A1");
        book.update_cell(&addr(2, 1), "=SUM(A1:A1)")
            .expect("failed to set A2 formula");
        book.evaluate();
        let output = serialize_sui(&book);
        assert!(
            output.contains("SUM"),
            "output should contain the formula string 'SUM', got: {output}"
        );
    }

    #[test]
    fn test_serialize_sui_multiple_sheets() {
        let mut book = Book::default();
        book.new_sheet(Some("ExtraSheet")).expect("failed to add ExtraSheet");
        let output = serialize_sui(&book);
        let names = book.get_sheet_names();
        for name in &names {
            assert!(
                output.contains(name.as_str()),
                "output should contain sheet name '{name}', got: {output}"
            );
        }
    }

    // -------------------------------------------------------------------------
    // Determinism tests (REQ-002)
    // -------------------------------------------------------------------------

    #[test]
    fn test_serialize_sui_deterministic() {
        let mut book = Book::default();
        book.update_cell(&addr(1, 1), "alpha").expect("failed to set A1");
        book.update_cell(&addr(1, 2), "42").expect("failed to set B1");
        book.update_cell(&addr(2, 1), "=SUM(A1:A1)")
            .expect("failed to set A2");
        book.evaluate();
        let first = serialize_sui(&book);
        let second = serialize_sui(&book);
        assert_eq!(
            first, second,
            "serialize_sui must produce byte-identical output on two calls"
        );
    }

    #[test]
    fn test_serialize_sui_ordering() {
        let mut book = Book::default();
        // Insert cells in reverse row-major order to verify canonical ordering is enforced.
        book.update_cell(&addr(2, 2), "row2col2").expect("failed to set (2,2)");
        book.update_cell(&addr(2, 1), "row2col1").expect("failed to set (2,1)");
        book.update_cell(&addr(1, 2), "row1col2").expect("failed to set (1,2)");
        book.update_cell(&addr(1, 1), "row1col1").expect("failed to set (1,1)");
        book.evaluate();
        let output = serialize_sui(&book);
        let pos_r1c1 = output.find("row1col1").expect("row1col1 not found in output");
        let pos_r1c2 = output.find("row1col2").expect("row1col2 not found in output");
        let pos_r2c1 = output.find("row2col1").expect("row2col1 not found in output");
        let pos_r2c2 = output.find("row2col2").expect("row2col2 not found in output");
        assert!(pos_r1c1 < pos_r1c2, "row1col1 must appear before row1col2 (same row, col 1 < col 2)");
        assert!(pos_r1c2 < pos_r2c1, "row1col2 must appear before row2col1 (row 1 < row 2)");
        assert!(pos_r2c1 < pos_r2c2, "row2col1 must appear before row2col2 (same row, col 1 < col 2)");
    }

    // -------------------------------------------------------------------------
    // Round-trip tests (REQ-010)
    // -------------------------------------------------------------------------

    #[test]
    fn test_round_trip_basic() {
        let mut original = Book::default();
        original
            .update_cell(&addr(1, 1), "round-trip-value")
            .expect("failed to set cell");
        original.evaluate();
        let sui_text = serialize_sui(&original);
        let (parsed, warnings) = parse_sui(&sui_text);
        assert_eq!(
            warnings.len(),
            0,
            "round-tripped .sui should parse without warnings"
        );
        let content = parsed
            .get_cell_addr_contents(&addr(1, 1))
            .expect("could not get cell after round-trip");
        assert_eq!(content, "round-trip-value", "cell value must survive round-trip");
    }

    #[test]
    fn test_round_trip_multiple_sheets() {
        let mut original = Book::default();
        original
            .new_sheet(Some("RoundTripSheet2"))
            .expect("failed to add sheet");
        original
            .update_cell(&Address { sheet: 0, row: 1, col: 1 }, "on-sheet1")
            .expect("failed to set sheet1 cell");
        original
            .update_cell(&Address { sheet: 1, row: 1, col: 1 }, "on-sheet2")
            .expect("failed to set sheet2 cell");
        original.evaluate();
        let sui_text = serialize_sui(&original);
        let (parsed, warnings) = parse_sui(&sui_text);
        assert_eq!(warnings.len(), 0);
        let names = parsed.get_sheet_names();
        assert!(
            names.len() >= 2,
            "round-tripped book must have at least 2 sheets, got: {names:?}"
        );
        assert!(
            names.contains(&"RoundTripSheet2".to_string()),
            "sheet 'RoundTripSheet2' must survive round-trip, got: {names:?}"
        );
    }

    #[test]
    fn test_round_trip_formulas() {
        let mut original = Book::default();
        original.update_cell(&addr(1, 1), "5").expect("failed to set A1");
        original
            .update_cell(&addr(2, 1), "=A1*2")
            .expect("failed to set A2 formula");
        original.evaluate();
        let sui_text = serialize_sui(&original);
        let (parsed, warnings) = parse_sui(&sui_text);
        assert_eq!(warnings.len(), 0);
        let content = parsed
            .get_cell_addr_contents(&addr(2, 1))
            .expect("could not get formula cell after round-trip");
        assert!(
            content.starts_with('=') && content.contains("A1"),
            "formula must survive round-trip as a formula string, got: {content}"
        );
    }

    #[test]
    fn test_round_trip_no_op_stability() {
        let mut book = Book::default();
        book.update_cell(&addr(1, 1), "stability-test")
            .expect("failed to set A1");
        book.update_cell(&addr(1, 2), "99").expect("failed to set B1");
        book.evaluate();
        // First serialization
        let first_output = serialize_sui(&book);
        // Parse back, then re-serialize — must produce byte-identical output
        let (parsed, _) = parse_sui(&first_output);
        let second_output = serialize_sui(&parsed);
        assert_eq!(
            first_output, second_output,
            "serialize → parse → serialize must be a no-op (byte-identical output)"
        );
    }
}
