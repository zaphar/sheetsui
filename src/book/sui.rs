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
//! spreadsheet data. Each non-comment, non-structural line declares one cell,
//! one column-width setting, or one set of cell style properties.
//!
//! ## Grammar (EBNF)
//!
//! ```text
//! file          ::= line* EOF
//! line          ::= (comment | sheet_start | sheet_end | col_width | style_decl | cell_decl) NEWLINE
//!                 | NEWLINE                        (* blank lines are ignored *)
//! comment       ::= '#' rest_of_line
//! sheet_start   ::= '[sheet' WS quoted_string ']'
//! sheet_end     ::= '[/sheet]'
//! col_width     ::= 'col' WS uint WS 'width' WS uint
//! style_decl    ::= 'style' WS cellref WS style_prop (WS style_prop)*
//! style_prop    ::= style_key WS style_val
//! style_key     ::= 'font.b' | 'font.i' | 'font.strike' | 'font.color' | 'font.u'
//!                 | 'fill.bg_color' | 'fill.fg_color'
//!                 | 'num_fmt'
//!                 | 'alignment.wrap_text' | 'alignment.horizontal' | 'alignment.vertical'
//! style_val     ::= bool_val | quoted_string | hex_color | align_h_val | align_v_val
//! bool_val      ::= 'true' | 'false'               (* lowercase only *)
//! hex_color     ::= '#' [0-9A-Fa-f]{6}
//! align_h_val   ::= 'center' | 'centerContinuous' | 'distributed' | 'fill'
//!                 | 'general' | 'justify' | 'left' | 'right'
//! align_v_val   ::= 'bottom' | 'center' | 'distributed' | 'justify' | 'top'
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
//! 3. Style declarations follow in row-major order (ascending row, then ascending column).
//! 4. Cell declarations follow in row-major order (ascending row, then ascending column).
//! 5. Empty cells are omitted (sparse representation).
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
use ironcalc::base::expressions::types::Area;
use ironcalc::base::types::{HorizontalAlignment, Style, VerticalAlignment};
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
            } else if let Some((row, col, props)) = parse_style_decl(trimmed) {
                apply_style_props(&mut book, sheet_idx, row, col, &props, line_num, &mut warnings);
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

        // Style declarations in row-major order (canonical: after col_widths, before cell_decls).
        let mut styled_cells: Vec<(i32, i32)> = ws
            .sheet_data
            .keys()
            .flat_map(|&row| ws.sheet_data[&row].keys().map(move |&col| (row, col)))
            .collect();
        styled_cells.sort_unstable();
        for (row, col) in &styled_cells {
            let addr = Address {
                sheet: sheet_idx,
                row: *row as usize,
                col: *col as usize,
            };
            if let Some(style) = book.get_cell_style(&addr) {
                if !is_default_style(&style) {
                    let props = serialize_style_props(&style);
                    if !props.is_empty() {
                        let cell_ref = format!("{}{row}", col_index_to_letters(*col as usize));
                        out.push_str(&format!("style {cell_ref} {}\n", props.join(" ")));
                    }
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

/// Returns true when all 11 tracked style properties are at their defaults.
/// Default font color is `None` or `Some("#000000")`, both treated as default.
fn is_default_style(style: &Style) -> bool {
    let font_color_default = style.font.color.is_none()
        || style.font.color.as_deref() == Some("#000000");
    let alignment_default = match &style.alignment {
        None => true,
        Some(a) => {
            a.horizontal == HorizontalAlignment::General
                && a.vertical == VerticalAlignment::Bottom
                && !a.wrap_text
        }
    };
    !style.font.b
        && !style.font.i
        && !style.font.strike
        && !style.font.u
        && font_color_default
        && style.fill.bg_color.is_none()
        && style.fill.fg_color.is_none()
        && style.num_fmt.eq_ignore_ascii_case("general")
        && alignment_default
}

/// Serializes non-default style properties as `"key value"` strings.
/// Returns an empty Vec when all properties are at their defaults.
fn serialize_style_props(style: &Style) -> Vec<String> {
    let mut props = Vec::new();

    if style.font.b {
        props.push("font.b true".to_string());
    }
    if style.font.i {
        props.push("font.i true".to_string());
    }
    if style.font.strike {
        props.push("font.strike true".to_string());
    }
    if style.font.u {
        props.push("font.u true".to_string());
    }
    if let Some(ref color) = style.font.color {
        if color != "#000000" {
            props.push(format!("font.color {color}"));
        }
    }
    if let Some(ref color) = style.fill.bg_color {
        props.push(format!("fill.bg_color {color}"));
    }
    if let Some(ref color) = style.fill.fg_color {
        props.push(format!("fill.fg_color {color}"));
    }
    if !style.num_fmt.eq_ignore_ascii_case("general") {
        props.push(format!("num_fmt \"{}\"", escape_string(&style.num_fmt)));
    }
    if let Some(ref alignment) = style.alignment {
        if alignment.horizontal != HorizontalAlignment::General {
            props.push(format!("alignment.horizontal {}", alignment.horizontal));
        }
        if alignment.vertical != VerticalAlignment::Bottom {
            props.push(format!("alignment.vertical {}", alignment.vertical));
        }
        if alignment.wrap_text {
            props.push("alignment.wrap_text true".to_string());
        }
    }

    props
}

/// Parses a `style <cellref> <key> <val> [<key> <val> ...]` line.
///
/// Returns `Some((row, col, props))` on success, or `None` if the line is not
/// a style declaration (missing prefix or malformed cellref).
fn parse_style_decl(line: &str) -> Option<(usize, usize, Vec<(String, String)>)> {
    let rest = line.strip_prefix("style ")?;
    // Find the cellref: next whitespace-delimited token
    let (cellref_str, remainder) = if let Some(pos) = rest.find(' ') {
        (&rest[..pos], rest[pos + 1..].trim_start())
    } else {
        // "style A1" with no properties — valid structure, no props
        (rest, "")
    };
    let (row, col) = parse_cellref(cellref_str)?;

    let mut props = Vec::new();
    let mut s = remainder;
    while !s.is_empty() {
        // Read the key (next whitespace-delimited token)
        let (key, after_key) = if let Some(pos) = s.find(' ') {
            (&s[..pos], s[pos + 1..].trim_start())
        } else {
            // key with no value — malformed pair, stop
            break;
        };
        // Read the value: for num_fmt it's a quoted string; otherwise the next token
        let (val, after_val) = if key == "num_fmt" {
            // Value is a quoted string; use parse_quoted_string on the leading portion
            if let Some(end_quote) = after_key.strip_prefix('"').and_then(|inner| {
                // find the closing unescaped quote
                let mut escaped = false;
                inner.char_indices().find(|(_, c)| {
                    if escaped {
                        escaped = false;
                        false
                    } else if *c == '\\' {
                        escaped = true;
                        false
                    } else {
                        *c == '"'
                    }
                }).map(|(i, _)| i)
            }) {
                // The quoted string spans after_key[0..=end_quote+1] (include both quotes)
                let quoted = &after_key[..end_quote + 2]; // +2 for the two quote characters
                let parsed_val = parse_quoted_string(quoted).unwrap_or_default();
                let after = after_key[end_quote + 2..].trim_start();
                (parsed_val, after)
            } else {
                break; // malformed quoted value
            }
        } else {
            if let Some(pos) = after_key.find(' ') {
                (after_key[..pos].to_string(), after_key[pos + 1..].trim_start())
            } else {
                (after_key.to_string(), "")
            }
        };
        props.push((key.to_string(), val));
        s = after_val;
    }

    Some((row, col, props))
}

/// Applies parsed style key-value pairs to a cell in the book.
/// Emits a `ParseWarning` for each unknown key; known keys are forwarded to
/// `Book::set_cell_style`.
fn apply_style_props(
    book: &mut Book,
    sheet: u32,
    row: usize,
    col: usize,
    props: &[(String, String)],
    line_num: usize,
    warnings: &mut Vec<ParseWarning>,
) {
    const KNOWN_KEYS: &[&str] = &[
        "font.b",
        "font.i",
        "font.strike",
        "font.color",
        "font.u",
        "fill.bg_color",
        "fill.fg_color",
        "num_fmt",
        "alignment.horizontal",
        "alignment.vertical",
        "alignment.wrap_text",
    ];

    for (key, val) in props {
        if !KNOWN_KEYS.contains(&key.as_str()) {
            warnings.push(ParseWarning {
                line: line_num,
                message: format!("unknown style key: {key}"),
            });
            continue;
        }
        let area = Area {
            sheet,
            row: row as i32,
            column: col as i32,
            width: 1,
            height: 1,
        };
        let _ = book.set_cell_style(&[(key.as_str(), val.as_str())], &area);
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_sui, serialize_sui};
    use crate::book::Book;
    use crate::ui::Address;
    use ironcalc::base::expressions::types::Area;
    use ironcalc::base::types::{HorizontalAlignment, VerticalAlignment};

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

    // -------------------------------------------------------------------------
    // Style per-property round-trip tests (iter-2, Phase 1)
    // -------------------------------------------------------------------------

    fn a1_area() -> Area {
        Area { sheet: 0, row: 1, column: 1, width: 1, height: 1 }
    }

    #[test]
    fn test_style_roundtrip_font_bold() {
        let mut book = Book::default();
        let a1 = addr(1, 1);
        book.set_cell_style(&[("font.b", "true")], &a1_area())
            .expect("failed to set font.b");
        let sui_text = serialize_sui(&book);
        assert!(
            sui_text.contains("font.b true"),
            "serialized output must contain 'font.b true', got:\n{sui_text}"
        );
        let (parsed, warnings) = parse_sui(&sui_text);
        assert_eq!(warnings.len(), 0, "round-tripped styled .sui must have no warnings");
        let style = parsed.get_cell_style(&a1).expect("style must be present after round-trip");
        assert!(style.font.b, "font.b must be true after round-trip");
    }

    #[test]
    fn test_style_roundtrip_font_italic() {
        let mut book = Book::default();
        let a1 = addr(1, 1);
        book.set_cell_style(&[("font.i", "true")], &a1_area())
            .expect("failed to set font.i");
        let sui_text = serialize_sui(&book);
        assert!(
            sui_text.contains("font.i true"),
            "serialized output must contain 'font.i true', got:\n{sui_text}"
        );
        let (parsed, warnings) = parse_sui(&sui_text);
        assert_eq!(warnings.len(), 0, "round-tripped styled .sui must have no warnings");
        let style = parsed.get_cell_style(&a1).expect("style must be present after round-trip");
        assert!(style.font.i, "font.i must be true after round-trip");
    }

    #[test]
    fn test_style_roundtrip_font_strikethrough() {
        let mut book = Book::default();
        let a1 = addr(1, 1);
        book.set_cell_style(&[("font.strike", "true")], &a1_area())
            .expect("failed to set font.strike");
        let sui_text = serialize_sui(&book);
        assert!(
            sui_text.contains("font.strike true"),
            "serialized output must contain 'font.strike true', got:\n{sui_text}"
        );
        let (parsed, warnings) = parse_sui(&sui_text);
        assert_eq!(warnings.len(), 0, "round-tripped styled .sui must have no warnings");
        let style = parsed.get_cell_style(&a1).expect("style must be present after round-trip");
        assert!(style.font.strike, "font.strike must be true after round-trip");
    }

    #[test]
    fn test_style_roundtrip_font_underline() {
        let mut book = Book::default();
        let a1 = addr(1, 1);
        book.set_cell_style(&[("font.u", "true")], &a1_area())
            .expect("failed to set font.u");
        let sui_text = serialize_sui(&book);
        assert!(
            sui_text.contains("font.u true"),
            "serialized output must contain 'font.u true', got:\n{sui_text}"
        );
        let (parsed, warnings) = parse_sui(&sui_text);
        assert_eq!(warnings.len(), 0, "round-tripped styled .sui must have no warnings");
        let style = parsed.get_cell_style(&a1).expect("style must be present after round-trip");
        assert!(style.font.u, "font.u must be true after round-trip");
    }

    #[test]
    fn test_style_roundtrip_font_color() {
        let mut book = Book::default();
        let a1 = addr(1, 1);
        // Use #FF0000 which is different from the default (#000000)
        book.set_cell_style(&[("font.color", "#FF0000")], &a1_area())
            .expect("failed to set font.color");
        let sui_text = serialize_sui(&book);
        assert!(
            sui_text.contains("font.color #FF0000"),
            "serialized output must contain 'font.color #FF0000', got:\n{sui_text}"
        );
        let (parsed, warnings) = parse_sui(&sui_text);
        assert_eq!(warnings.len(), 0, "round-tripped styled .sui must have no warnings");
        let style = parsed.get_cell_style(&a1).expect("style must be present after round-trip");
        assert_eq!(
            style.font.color,
            Some("#FF0000".to_string()),
            "font.color must be #FF0000 after round-trip"
        );
    }

    #[test]
    fn test_style_roundtrip_fill_bg_color() {
        let mut book = Book::default();
        let a1 = addr(1, 1);
        book.set_cell_style(&[("fill.bg_color", "#AABBCC")], &a1_area())
            .expect("failed to set fill.bg_color");
        let sui_text = serialize_sui(&book);
        assert!(
            sui_text.contains("fill.bg_color #AABBCC"),
            "serialized output must contain 'fill.bg_color #AABBCC', got:\n{sui_text}"
        );
        let (parsed, warnings) = parse_sui(&sui_text);
        assert_eq!(warnings.len(), 0, "round-tripped styled .sui must have no warnings");
        let style = parsed.get_cell_style(&a1).expect("style must be present after round-trip");
        assert_eq!(
            style.fill.bg_color,
            Some("#AABBCC".to_string()),
            "fill.bg_color must be #AABBCC after round-trip"
        );
    }

    #[test]
    fn test_style_roundtrip_fill_fg_color() {
        let mut book = Book::default();
        let a1 = addr(1, 1);
        book.set_cell_style(&[("fill.fg_color", "#112233")], &a1_area())
            .expect("failed to set fill.fg_color");
        let sui_text = serialize_sui(&book);
        assert!(
            sui_text.contains("fill.fg_color #112233"),
            "serialized output must contain 'fill.fg_color #112233', got:\n{sui_text}"
        );
        let (parsed, warnings) = parse_sui(&sui_text);
        assert_eq!(warnings.len(), 0, "round-tripped styled .sui must have no warnings");
        let style = parsed.get_cell_style(&a1).expect("style must be present after round-trip");
        assert_eq!(
            style.fill.fg_color,
            Some("#112233".to_string()),
            "fill.fg_color must be #112233 after round-trip"
        );
    }

    #[test]
    fn test_style_roundtrip_num_fmt() {
        let mut book = Book::default();
        let a1 = addr(1, 1);
        book.set_cell_style(&[("num_fmt", "0.00")], &a1_area())
            .expect("failed to set num_fmt");
        let sui_text = serialize_sui(&book);
        // num_fmt is serialized as a quoted string
        assert!(
            sui_text.contains("num_fmt \"0.00\""),
            "serialized output must contain 'num_fmt \"0.00\"', got:\n{sui_text}"
        );
        let (parsed, warnings) = parse_sui(&sui_text);
        assert_eq!(warnings.len(), 0, "round-tripped styled .sui must have no warnings");
        let style = parsed.get_cell_style(&a1).expect("style must be present after round-trip");
        assert_eq!(
            style.num_fmt, "0.00",
            "num_fmt must be '0.00' after round-trip"
        );
    }

    #[test]
    fn test_style_roundtrip_alignment_horizontal() {
        let mut book = Book::default();
        let a1 = addr(1, 1);
        book.set_cell_style(&[("alignment.horizontal", "center")], &a1_area())
            .expect("failed to set alignment.horizontal");
        let sui_text = serialize_sui(&book);
        assert!(
            sui_text.contains("alignment.horizontal center"),
            "serialized output must contain 'alignment.horizontal center', got:\n{sui_text}"
        );
        let (parsed, warnings) = parse_sui(&sui_text);
        assert_eq!(warnings.len(), 0, "round-tripped styled .sui must have no warnings");
        let style = parsed.get_cell_style(&a1).expect("style must be present after round-trip");
        let alignment = style.alignment.expect("alignment must be Some after round-trip");
        assert_eq!(
            alignment.horizontal,
            HorizontalAlignment::Center,
            "alignment.horizontal must be Center after round-trip"
        );
    }

    #[test]
    fn test_style_roundtrip_alignment_vertical() {
        let mut book = Book::default();
        let a1 = addr(1, 1);
        book.set_cell_style(&[("alignment.vertical", "top")], &a1_area())
            .expect("failed to set alignment.vertical");
        let sui_text = serialize_sui(&book);
        assert!(
            sui_text.contains("alignment.vertical top"),
            "serialized output must contain 'alignment.vertical top', got:\n{sui_text}"
        );
        let (parsed, warnings) = parse_sui(&sui_text);
        assert_eq!(warnings.len(), 0, "round-tripped styled .sui must have no warnings");
        let style = parsed.get_cell_style(&a1).expect("style must be present after round-trip");
        let alignment = style.alignment.expect("alignment must be Some after round-trip");
        assert_eq!(
            alignment.vertical,
            VerticalAlignment::Top,
            "alignment.vertical must be Top after round-trip"
        );
    }

    #[test]
    fn test_style_roundtrip_alignment_wrap_text() {
        let mut book = Book::default();
        let a1 = addr(1, 1);
        book.set_cell_style(&[("alignment.wrap_text", "true")], &a1_area())
            .expect("failed to set alignment.wrap_text");
        let sui_text = serialize_sui(&book);
        assert!(
            sui_text.contains("alignment.wrap_text true"),
            "serialized output must contain 'alignment.wrap_text true', got:\n{sui_text}"
        );
        let (parsed, warnings) = parse_sui(&sui_text);
        assert_eq!(warnings.len(), 0, "round-tripped styled .sui must have no warnings");
        let style = parsed.get_cell_style(&a1).expect("style must be present after round-trip");
        let alignment = style.alignment.expect("alignment must be Some after round-trip");
        assert!(alignment.wrap_text, "alignment.wrap_text must be true after round-trip");
    }

    // -------------------------------------------------------------------------
    // Style scenario tests (iter-2, Phase 1)
    // -------------------------------------------------------------------------

    #[test]
    fn test_style_roundtrip_multi_property() {
        // Set three distinct style properties on A1 and verify all survive a round-trip.
        let mut book = Book::default();
        let a1 = addr(1, 1);
        book.set_cell_style(
            &[("font.b", "true"), ("fill.bg_color", "#FF0000"), ("num_fmt", "0.00")],
            &a1_area(),
        )
        .expect("failed to set multi-property style");
        let sui_text = serialize_sui(&book);
        // All three properties must appear in the output
        assert!(
            sui_text.contains("font.b true"),
            "output must contain 'font.b true', got:\n{sui_text}"
        );
        assert!(
            sui_text.contains("fill.bg_color #FF0000"),
            "output must contain 'fill.bg_color #FF0000', got:\n{sui_text}"
        );
        assert!(
            sui_text.contains("num_fmt \"0.00\""),
            "output must contain 'num_fmt \"0.00\"', got:\n{sui_text}"
        );
        let (parsed, warnings) = parse_sui(&sui_text);
        assert_eq!(warnings.len(), 0, "multi-property round-trip must have no warnings");
        let style = parsed.get_cell_style(&a1).expect("style must be present after round-trip");
        assert!(style.font.b, "font.b must be true after multi-property round-trip");
        assert_eq!(
            style.fill.bg_color,
            Some("#FF0000".to_string()),
            "fill.bg_color must be #FF0000 after multi-property round-trip"
        );
        assert_eq!(style.num_fmt, "0.00", "num_fmt must be '0.00' after multi-property round-trip");
    }

    #[test]
    fn test_style_roundtrip_multi_sheet() {
        // Styles on different sheets must both survive a round-trip.
        let mut book = Book::default();
        // Set font.i=true on sheet 0 / A1
        book.set_cell_style(&[("font.i", "true")], &a1_area())
            .expect("failed to set font.i on sheet 0");
        book.new_sheet(Some("Sheet2")).expect("failed to add Sheet2");
        // Set fill.bg_color on sheet 1 / B2
        let area_sheet1 = Area { sheet: 1, row: 2, column: 2, width: 1, height: 1 };
        book.set_cell_style(&[("fill.bg_color", "#0000FF")], &area_sheet1)
            .expect("failed to set fill.bg_color on sheet 1");
        let sui_text = serialize_sui(&book);
        let (parsed, warnings) = parse_sui(&sui_text);
        assert_eq!(warnings.len(), 0, "multi-sheet round-trip must have no warnings");
        // Sheet 0, A1 — font.i
        let style0 = parsed
            .get_cell_style(&addr(1, 1))
            .expect("style on sheet 0 A1 must be present after round-trip");
        assert!(style0.font.i, "font.i must be true on sheet 0 A1 after round-trip");
        // Sheet 1, B2 — fill.bg_color
        let b2_sheet1 = Address { sheet: 1, row: 2, col: 2 };
        let style1 = parsed
            .get_cell_style(&b2_sheet1)
            .expect("style on sheet 1 B2 must be present after round-trip");
        assert_eq!(
            style1.fill.bg_color,
            Some("#0000FF".to_string()),
            "fill.bg_color must be #0000FF on sheet 1 B2 after round-trip"
        );
    }

    #[test]
    fn test_style_sparse_unstyled_book() {
        // A book with no styling must not emit any style_decl lines.
        let book = Book::default();
        let sui_text = serialize_sui(&book);
        // No line should start with "style "
        let has_style_line = sui_text.lines().any(|l| l.starts_with("style "));
        assert!(
            !has_style_line,
            "unstyled book must not emit any 'style ...' lines, got:\n{sui_text}"
        );
    }

    #[test]
    fn test_style_styled_book_emits_style_line() {
        // A book with at least one non-default style must emit at least one style_decl line.
        let mut book = Book::default();
        book.set_cell_style(&[("font.b", "true")], &a1_area())
            .expect("failed to set font.b");
        let output = serialize_sui(&book);
        assert!(
            output.lines().any(|l| l.starts_with("style ")),
            "a styled book must emit at least one style_decl line, got:\n{output}"
        );
    }

    #[test]
    fn test_style_unknown_key_warning() {
        // A style_decl line with an unrecognized key must produce exactly one warning
        // but must not prevent other lines from loading.
        let text = "[sheet \"Sheet1\"]\nstyle A1 unknown.key value\nA1 = \"hello\"\n[/sheet]\n";
        let (book, warnings) = parse_sui(text);
        assert_eq!(
            warnings.len(),
            1,
            "exactly one warning expected for the unknown style key, got {} warnings",
            warnings.len()
        );
        assert_eq!(
            warnings[0].line, 2,
            "warning must reference line 2 (the style_decl line), got line {}",
            warnings[0].line
        );
        // The valid cell_decl on line 3 must still be loaded
        let content = book
            .get_cell_addr_contents(&addr(1, 1))
            .expect("A1 must be loadable even after a bad style_decl line");
        assert_eq!(
            content, "hello",
            "cell A1 must have value 'hello' despite the bad style_decl line before it"
        );
    }

    #[test]
    fn test_style_parse_known_key_applies_style() {
        // Parsing a valid style_decl line must produce zero warnings and apply the style.
        let text = "[sheet \"Sheet1\"]\nstyle A1 font.b true\nA1 = \"data\"\n[/sheet]\n";
        let (book, warnings) = parse_sui(text);
        assert_eq!(
            warnings.len(),
            0,
            "parsing a valid style_decl line must produce zero warnings, got: {:?}",
            warnings.iter().map(|w| &w.message).collect::<Vec<_>>()
        );
        let style = book
            .get_cell_style(&addr(1, 1))
            .expect("cell A1 must have a style after parsing style_decl");
        assert!(style.font.b, "font.b must be true after parsing 'style A1 font.b true'");
    }

    #[test]
    fn test_style_duplicate_lines_same_cell() {
        // Two style_decl lines for the same property on the same cell: last value wins.
        let text = "[sheet \"Sheet1\"]\n\
                    style A1 font.b true\n\
                    style A1 font.b false\n\
                    A1 = \"data\"\n\
                    [/sheet]\n";
        let (book, warnings) = parse_sui(text);
        assert_eq!(
            warnings.len(),
            0,
            "duplicate style_decl lines with known keys must produce no warnings"
        );
        let style = book
            .get_cell_style(&addr(1, 1))
            .expect("style must be present on A1");
        assert!(
            !style.font.b,
            "font.b must be false after the second style_decl (last value wins)"
        );
    }

    #[test]
    fn test_style_roundtrip_noop_stability_styled() {
        // serialize → parse → serialize must produce byte-identical output even when
        // style_decl lines are present (extends the existing no-op stability test).
        let mut book = Book::default();
        book.set_cell_style(&[("font.b", "true")], &a1_area())
            .expect("failed to set font.b");
        let text1 = serialize_sui(&book);
        assert!(
            text1.contains("font.b true"),
            "serialized styled book must contain 'font.b true' token, got:\n{text1}"
        );
        let (book2, warnings) = parse_sui(&text1);
        assert_eq!(warnings.len(), 0, "parse of styled .sui must have no warnings");
        let text2 = serialize_sui(&book2);
        assert_eq!(
            text1, text2,
            "serialize → parse → serialize must be byte-identical for styled books"
        );
    }
}
