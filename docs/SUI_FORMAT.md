# .sui File Format Specification

The `.sui` format is sheetui's native spreadsheet format. It is a declarative,
sparse, human-readable plain-text format designed to be easy to read in any
text editor, friendly to version control, and straightforward to parse.

This document is the complete format reference for `.sui` files.

---

## Table of Contents

1. [Overview](#overview)
2. [EBNF Grammar](#ebnf-grammar)
3. [Line Types](#line-types)
   - [Blank lines and Comments](#blank-lines-and-comments)
   - [Sheet start and end](#sheet-start-and-end)
   - [Column width declaration](#column-width-declaration-col_width)
   - [Style declaration](#style-declaration-style_decl)
   - [Cell declaration](#cell-declaration-cell_decl)
4. [Style Keys Reference](#style-keys-reference)
5. [Serialization Order](#serialization-order)
6. [Annotated Example](#annotated-example)
7. [Parser Behavior](#parser-behavior)
8. [Backward Compatibility](#backward-compatibility)
9. [Hand-Editing Guidelines](#hand-editing-guidelines)

---

## Overview

A `.sui` file is a sequence of lines. Each meaningful line belongs to one of
five categories: comment, sheet boundary, column width, style declaration, or
cell declaration. All data lines must appear inside a sheet block. The format
is sparse: cells with no value and columns with default width are omitted.

---

## EBNF Grammar

```text
file          ::= line* EOF
line          ::= (comment | sheet_start | sheet_end | col_width | style_decl | cell_decl) NEWLINE
                | NEWLINE                        (* blank lines are ignored *)
comment       ::= '#' rest_of_line
sheet_start   ::= '[sheet' WS quoted_string ']'
sheet_end     ::= '[/sheet]'
col_width     ::= 'col' WS uint WS 'width' WS uint
style_decl    ::= 'style' WS cellref WS style_prop (WS style_prop)*
style_prop    ::= style_key WS style_val
style_key     ::= 'font.b' | 'font.i' | 'font.strike' | 'font.color' | 'font.u'
                | 'fill.bg_color' | 'fill.fg_color'
                | 'num_fmt'
                | 'alignment.wrap_text' | 'alignment.horizontal' | 'alignment.vertical'
style_val     ::= bool_val | quoted_string | hex_color | align_h_val | align_v_val
bool_val      ::= 'true' | 'false'               (* lowercase only *)
hex_color     ::= '#' [0-9A-Fa-f]{6}
align_h_val   ::= 'center' | 'centerContinuous' | 'distributed' | 'fill'
                | 'general' | 'justify' | 'left' | 'right'
align_v_val   ::= 'bottom' | 'center' | 'distributed' | 'justify' | 'top'
cell_decl     ::= cellref WS '=' WS value
cellref       ::= [A-Z]+ [1-9][0-9]*            (* standard A1 notation, 1-based *)
value         ::= string | number | boolean | formula
string        ::= '"' char* '"'                  (* escapes: \" \\ \n *)
number        ::= '-'? [0-9]+ ('.' [0-9]+)?
boolean       ::= 'true' | 'false'               (* case-insensitive *)
formula       ::= '=' rest_of_line               (* everything after '=' is the formula *)
quoted_string ::= '"' char* '"'
uint          ::= [0-9]+
WS            ::= ' '+
```

---

## Line Types

### Blank lines and Comments

Blank lines (containing only whitespace) and lines whose first non-whitespace
character is `#` are ignored by the parser. Use comments to annotate your
spreadsheet.

```
# This is a comment.
# Comments may appear anywhere in the file, including between sheet blocks.

[sheet "Budget"]
# Column A holds month labels
A1 = "January"
[/sheet]
```

### Sheet start and end

Every sheet is wrapped in a pair of boundary lines:

```
[sheet "Sheet Name"]
... declarations ...
[/sheet]
```

- The sheet name is a double-quoted string using the same escape rules as
  cell string values (`\"`, `\\`, `\n`).
- Sheet names must be unique within a workbook.
- Multiple sheet blocks may appear in sequence; they are loaded in file order.
- Any data line that appears outside a sheet block is recorded as a
  `ParseWarning` and skipped.

### Column width declaration (`col_width`)

Sets the display width for a column within the current sheet:

```
col <column-number> width <characters>
```

- `<column-number>` is a 1-based positive integer (column A = 1, B = 2, ...).
- `<characters>` is a positive integer representing the column width in
  character-cell units.
- Only columns whose width differs from the application default are written
  during serialization.

Example:

```
col 1 width 20
col 3 width 8
```

### Style declaration (`style_decl`)

Sets one or more style properties on a single cell:

```
style <cellref> <key> <value> [<key> <value> ...]
```

- `<cellref>` uses standard A1 notation (see [Cell Declarations](#cell-declaration-cell_decl)).
- Each `<key> <value>` pair sets one style property on that cell.
- Multiple properties may be specified on a single line, separated by spaces.
- Only cells with at least one non-default style property are written during
  serialization.

Example:

```
style A1 font.b true font.color #FF0000
style B3 fill.bg_color #FFFFCC alignment.horizontal center
```

See the [Style Keys Reference](#style-keys-reference) for the complete list of
supported keys and their accepted values.

### Cell declaration (`cell_decl`)

Sets the value of a single cell:

```
<cellref> = <value>
```

The cell reference uses standard spreadsheet notation: one or more uppercase
letters for the column followed by a positive integer for the row (minimum 1).

```
A1   B2   Z10   AA1   AB5
```

Column letters map to column numbers: `A` = 1, `Z` = 26, `AA` = 27, `AB` = 28,
and so on. Row numbers start at 1.

**Value types:**

| Value type | Syntax                     | Example                    |
|------------|----------------------------|----------------------------|
| String     | `"..."` (double-quoted)    | `A1 = "hello world"`       |
| Number     | Plain decimal literal      | `B1 = 42` or `B2 = 3.14`  |
| Boolean    | `true` or `false`          | `C1 = true`                |
| Formula    | Starts with `=`            | `D1 = =SUM(A1:A3)`         |

String escape sequences:

| Escape | Character    |
|--------|--------------|
| `\"`   | Double quote |
| `\\`   | Backslash    |
| `\n`   | Newline      |

Boolean values are case-insensitive on input (`True`, `TRUE`, and `true` are
all accepted); the serializer always writes lowercase (`true` / `false`).

---

## Style Keys Reference

The following 11 style properties are supported. The table lists each key, its
accepted value type, the default value (the value that is omitted during
serialization), and a usage example.

| Key                      | Value type     | Default          | Example                                |
|--------------------------|----------------|------------------|----------------------------------------|
| `font.b`                 | `bool_val`     | `false`          | `style A1 font.b true`                 |
| `font.i`                 | `bool_val`     | `false`          | `style A1 font.i true`                 |
| `font.strike`            | `bool_val`     | `false`          | `style A1 font.strike true`            |
| `font.u`                 | `bool_val`     | `false`          | `style A1 font.u true`                 |
| `font.color`             | `hex_color`    | `#000000` (black)| `style A1 font.color #FF0000`          |
| `fill.bg_color`          | `hex_color`    | none             | `style A1 fill.bg_color #FFFFCC`       |
| `fill.fg_color`          | `hex_color`    | none             | `style A1 fill.fg_color #CCCCFF`       |
| `num_fmt`                | `quoted_string`| `"General"`      | `style A1 num_fmt "0.00%"`             |
| `alignment.wrap_text`    | `bool_val`     | `false`          | `style A1 alignment.wrap_text true`    |
| `alignment.horizontal`   | `align_h_val`  | `general`        | `style A1 alignment.horizontal center` |
| `alignment.vertical`     | `align_v_val`  | `bottom`         | `style A1 alignment.vertical top`      |

### `alignment.horizontal` valid values

| Value               | Meaning                                          |
|---------------------|--------------------------------------------------|
| `general`           | Default alignment (left for text, right for numbers) |
| `left`              | Align content to the left edge of the cell       |
| `center`            | Center content horizontally                      |
| `right`             | Align content to the right edge of the cell      |
| `fill`              | Repeat content to fill the cell width            |
| `justify`           | Justify multi-line text between left and right   |
| `centerContinuous`  | Center across adjacent unoccupied cells          |
| `distributed`       | Distribute content evenly across the cell width  |

### `alignment.vertical` valid values

| Value         | Meaning                                          |
|---------------|--------------------------------------------------|
| `bottom`      | Default; align content to the bottom of the cell |
| `top`         | Align content to the top of the cell             |
| `center`      | Center content vertically                        |
| `justify`     | Justify multi-line text between top and bottom   |
| `distributed` | Distribute content evenly across the cell height |

### Color values

`hex_color` values are a `#` followed by exactly six hexadecimal digits (case-
insensitive). Both uppercase and lowercase hex digits are accepted on input.
The serializer always writes uppercase hex digits as produced by the underlying
ironcalc library.

Examples: `#FF0000` (red), `#00FF00` (green), `#ffffcc` (light yellow).

### Number format strings

`num_fmt` accepts any format string recognized by the ironcalc engine (a
superset of the Excel number format language). The value is a double-quoted
string. Common examples:

| Format string   | Renders as               |
|-----------------|--------------------------|
| `"General"`     | Default (auto-detect)    |
| `"0"`           | Integer                  |
| `"0.00"`        | Two decimal places       |
| `"0.00%"`       | Percentage               |
| `"#,##0"`       | Thousands separator      |
| `"YYYY-MM-DD"`  | ISO date                 |

---

## Serialization Order

When sheetui writes a `.sui` file the output is deterministic: given the same
workbook state, two calls always produce byte-identical output.

Canonical ordering within a file:

1. Sheets are written in workbook index order.
2. Within each sheet, `col` width declarations come first, in ascending
   column-index order.
3. Style declarations follow in row-major order (ascending row, then ascending
   column).
4. Cell declarations follow in row-major order (ascending row, then ascending
   column).
5. Empty cells are omitted (sparse representation).

---

## Annotated Example

The following example shows all line types together in a single `.sui` file.
Inline comments explain each construct.

```
# -----------------------------------------------------------------------
# Example .sui workbook — two sheets demonstrating all line types
# -----------------------------------------------------------------------

[sheet "Summary"]                       # Sheet block start — name is a quoted string

col 1 width 20                          # Column A is 20 characters wide
col 2 width 10                          # Column B is 10 characters wide

# Style declarations always appear after col_width lines and before cell
# declarations.  Each line targets one cell and may carry multiple
# key-value pairs.

style A1 font.b true font.color #1F4E79         # Bold, dark-blue font
style B1 fill.bg_color #FFFFCC                  # Light-yellow background
style C1 alignment.horizontal right             # Right-align
style A2 font.i true alignment.wrap_text true   # Italic, text wraps in cell
style B2 num_fmt "0.00%"                        # Percentage format
style C2 alignment.vertical top                 # Vertical top alignment

# Cell declarations follow in row-major order.

A1 = "Month"                            # String value (double-quoted)
B1 = "Amount"
C1 = "Status"
A2 = "January"
B2 = 0.1234                             # Number value (stored as-is)
C2 = true                               # Boolean value
A3 = "February"
B3 = =B2*1.05                           # Formula — everything after '=' is the formula
C3 = false

[/sheet]                                # Sheet block end

[sheet "Notes"]                         # Second sheet — no col_width or style lines needed
A1 = "See Summary tab for data."
[/sheet]
```

---

## Parser Behavior

The `.sui` parser is lenient by design (REQ-003):

- Lines that cannot be parsed are **skipped** and recorded as `ParseWarning`
  entries rather than causing a load failure.
- Valid lines before and after an invalid line are still loaded.
- A file in which every line is invalid results in an empty workbook, not an
  error.
- Warnings include the 1-based line number and a human-readable description of
  the problem.

Currently, `ParseWarning` entries are stored internally and not displayed in
the terminal interface. If a workbook loads with missing data, verify the file
follows the syntax described in this document.

**Unknown style keys** on an otherwise well-formed `style` line are recorded as
warnings and skipped; other recognized properties on the same line are still
applied.

---

## Backward Compatibility

The `style_decl` production was added in iteration 2 of the `.sui` format.

A parser that predates this extension will not recognize lines beginning with
`style` and will record each such line as a `ParseWarning`. It will **not**
abort loading: cell values and column widths from the same file are loaded
correctly. The workbook will be missing any styling information, but all data
will be intact.

This means `.sui` files that include `style_decl` lines are forward-readable
by older sheetui versions — data is never lost, only styling is silently
discarded.

Summary:

| Parser version   | Behavior on `style_decl` lines                         |
|------------------|-------------------------------------------------------|
| Iteration 2+     | Parses and applies style properties                    |
| Pre-iteration 2  | Records a `ParseWarning`, skips the line, loads data  |

---

## Hand-Editing Guidelines

Because `.sui` is plain text you can edit it in any editor. Keep these rules in
mind:

- Every sheet block must start with `[sheet "Name"]` and end with `[/sheet]`.
- Sheet names are double-quoted strings. Use `\"` to include a literal quote
  character in the name.
- Column width declarations must appear before style declarations, which must
  appear before cell declarations within the same sheet block.
- Cell references must use uppercase letters only (e.g., `A1`, not `a1`).
- Cell declarations require a space on each side of `=` (e.g., `A1 = "hello"`,
  not `A1="hello"`).
- String values must be enclosed in double quotes.
- Formulas must start with `=` after the `= ` separator (e.g., `A1 = =SUM(A1:A3)`).
- Style declarations use the form `style <cellref> <key> <value>`. Multiple
  key-value pairs may be placed on the same line.
- Boolean style values (`font.b`, `font.i`, etc.) must be exactly `true` or
  `false` in lowercase.
- Color values must be a `#` followed by exactly six hexadecimal digits.
- Any line that does not match a recognized pattern is silently skipped and
  recorded as a parse warning.
