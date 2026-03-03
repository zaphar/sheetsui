# File Formats

sheetui supports two file formats. The format is always determined by the file
extension when opening or saving.

| Extension | Format | Notes |
|-----------|--------|-------|
| `.xlsx`   | Excel-compatible | Use for sharing with other applications |
| `.sui` (or any other extension) | sheetui native | Default for new workbooks; human-readable |

## The `.sui` Format

`.sui` is sheetui's native plain-text spreadsheet format. It is designed to be
readable in any text editor and friendly to version control (each cell is one
line, making diffs easy to read).

For the complete format specification including the EBNF grammar, style key
reference, and annotated examples, see [SUI_FORMAT.md](./SUI_FORMAT.md).

### File Structure

A `.sui` file consists of one or more **sheet blocks**. Each block starts with
a `[sheet "Name"]` header and ends with `[/sheet]`. Inside a block you can
declare column widths, cell style properties, and cell values.

```
# This is a comment — comments begin with # and are ignored on load.

[sheet "Sheet1"]
col 1 width 15
col 2 width 20
style A1 font.b true font.color #1F4E79
A1 = "hello world"
B1 = 42
C1 = true
A2 = =SUM(A1:A1)
[/sheet]

[sheet "Sheet2"]
A1 = "another sheet"
[/sheet]
```

Blank lines and lines beginning with `#` are ignored.

### Cell References

Cells are addressed in standard spreadsheet notation: one or more uppercase
letters for the column followed by a positive integer for the row.

```
A1   B2   Z10   AA1   AB5
```

Column `A` is column 1, `Z` is column 26, `AA` is column 27, and so on.
Row numbers start at 1.

### Cell Values

Each cell declaration has the form `<ref> = <value>`:

| Value type | Syntax | Example |
|------------|--------|---------|
| String     | `"..."` (double-quoted) | `A1 = "hello"` |
| Number     | Plain decimal | `B1 = 42` or `B2 = 3.14` |
| Boolean    | `true` or `false` | `C1 = true` |
| Formula    | Starts with `=` | `D1 = =SUM(A1:A3)` |

String values may use the following escape sequences:

| Escape | Character |
|--------|-----------|
| `\"` | Double quote |
| `\\` | Backslash |
| `\n` | Newline |

### Column Widths

Column width declarations set the display width (in character columns) for a
given column within the current sheet:

```
col <column-number> width <characters>
```

Column numbers are 1-based (column A = 1, column B = 2, …). Only columns
whose width differs from the default are written to the file.

### Serialization Order

When sheetui writes a `.sui` file the output is deterministic:

1. Sheets appear in workbook index order.
2. Within each sheet, `col` width declarations come first, in ascending column
   order.
3. Style declarations follow in row-major order: ascending row, then ascending
   column within each row.
4. Cell declarations follow in row-major order: ascending row, then ascending
   column within each row.
5. Empty cells are omitted (the format is sparse).

### Parse Warnings

If a line in a `.sui` file cannot be recognised, sheetui skips that line,
records a warning internally, and continues loading the rest of the file. Valid
lines before and after an invalid line are still loaded. A completely invalid
file results in an empty workbook.

Parse warnings are currently stored internally and not displayed in the terminal
interface. If a file loads incorrectly, check that it follows the syntax
described above.

### Hand-Editing `.sui` Files

Because `.sui` is plain text you can edit it in any editor. Keep these rules in
mind:

* Every sheet block must start with `[sheet "Name"]` and end with `[/sheet]`.
* Sheet names are quoted and may not contain unescaped double-quote characters.
* Cells must use the `<ref> = <value>` form with a single space on each side of
  the `=`.
* Strings must be enclosed in double quotes.
* Formulas must begin with `=` (e.g. `=SUM(A1:A3)`).
* Any line that does not match one of the recognised patterns is silently
  skipped and recorded as a parse warning.

## The `.xlsx` Format

`.xlsx` files are read and written using the [ironcalc](https://docs.ironcalc.com/)
library, which provides Excel-compatible spreadsheet support. Formulas, multiple
sheets, and cell styles are all preserved.

Use `.xlsx` when you need to share a workbook with Excel, LibreOffice, or
another spreadsheet application.

**Note:** Cell styles (row/column background colors set with `color-rows`,
`color-columns`, or `color-cell`) are preserved in both `.xlsx` and `.sui`
files. When saving as `.sui`, style properties are written as `style`
declarations. See [SUI_FORMAT.md](./SUI_FORMAT.md) for the full list of
supported style properties.

## Format Auto-Detection

sheetui determines the format from the file extension:

* `.xlsx` → Excel format
* Anything else → `.sui` format

If you open a file with an unrecognised extension (such as `.csv`) sheetui will
attempt to parse it as `.sui`. To explicitly choose a format, use the
appropriate extension when saving with `:w <path>`.

## Choosing a Format

| Use case | Recommended format |
|----------|--------------------|
| Personal spreadsheets | `.sui` |
| Version-controlled spreadsheets | `.sui` |
| Sharing with Excel / LibreOffice | `.xlsx` |
| Preserving cell styling (within sheetui) | `.sui` |
| Preserving cell styling (cross-application) | `.xlsx` |
