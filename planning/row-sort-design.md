# Design: Sort Rows

Status: accepted 2026-10-25 — decisions D1–D7.

## User

A person who has a table in a sheet (often a loaded CSV with a header row)
and wants to put the rows in order by one or more columns.

## What sort does

Sort puts a set of rows in order by the values in one or more key columns.
All cells in a row move together. Nothing outside the sorted rows changes
position.

## Flow

1. The user selects the rows to sort. Two ways:
   - Make a range selection (`v` or `Ctrl-r`, then the spacebar twice).
   - Make no selection. Sort then uses the full used area of the sheet.
2. The user puts the cursor in the key column.
3. The user types `:sort` (descending) or `:sort asc` (ascending), or gives
   one or more key columns (see Command form).
4. The rows change order. The cursor stays on the same cell address.

This matches `:color-cell`, which uses the last range selection, or the
current cell when there is no selection.

### Command form

```
:sort [skip <N>] [asc|desc]
:sort [skip <N>] <column> [asc|desc] [<column> [asc|desc] ...]
```

| Command | Skipped rows | Keys |
|---------|--------------|------|
| `:sort` | none | cursor's column, desc |
| `:sort asc` | none | cursor's column, asc |
| `:sort B` | none | B desc |
| `:sort B C` | none | B desc, then C desc |
| `:sort B asc C` | none | B asc, then C desc |
| `:sort B C asc` | none | B desc, then C asc |
| `:sort skip 1 B asc C` | 1 | B asc, then C desc |
| `:sort skip 3 asc` | 3 | cursor's column, asc |

- `skip <N>` keeps the first N rows of the sort area in place. These are
  header rows. The sort area is the range selection, or the used area of the
  sheet when there is no selection. So with no selection, `skip 2` keeps
  rows 1 and 2; with a selection from row 5, `skip 2` keeps rows 5 and 6.
- `skip` must come first, right after `:sort`, and N is required.
  `skip 0` is allowed and skips nothing.
- Each key is a column letter, with an optional direction after it. `desc` is
  the default (D6).
- A direction applies only to the column just before it. `:sort B C asc`
  sorts B descending and C ascending, not both ascending. The help text must
  say this.
- Rows sort by the first key. Rows with equal first keys sort by the second
  key, and so on.
- Column letters and direction words are not case-sensitive.
- The words `asc` and `desc` are always directions, never column names. So
  column `ASC` cannot be a key by name. To sort by it, put the cursor on it
  and type `:sort`.

## Cases

| Case | Behavior |
|------|----------|
| Empty sheet | Do nothing. No error. |
| One row | Do nothing. |
| A key column outside the range | Error dialog: "Key column B is outside the selected range." Nothing moves. |
| Same column given two times (`:sort B C B`) | Error dialog: "Column B is given more than one time." Nothing moves. |
| Direction with no column before it (`:sort B desc desc`) | Error dialog with the usage line. Nothing moves. |
| Bad argument (`:sort sideways`) | Error dialog with the usage line. Nothing moves. |
| Header rows | `skip <N>` keeps the first N rows of the sort area in place (D3). |
| `skip` N is equal to or more than the rows in the sort area | Error dialog: "All N rows are skipped; nothing to sort." Nothing moves. |
| `skip` with no number, or not first (`:sort B skip 1`) | Error dialog with the usage line. Nothing moves. |
| Equal keys | Rows keep their old order (stable sort). |
| Blank key cells | Go to the bottom in both `asc` and `desc`. |
| Mixed types | Numbers before text, text before booleans, booleans before errors (the Excel order). Text compares without case. |
| Large sheet (10,000+ rows) | Must finish without a visible stall. Pause evaluation during the sort and evaluate one time at the end. |
| Sort area overlaps merged cells | Error dialog: "Cannot sort a range that contains merged cells." Nothing moves. |
| Sort area holds an array formula or its spill cells | Error dialog: "Cannot sort a range that contains array formulas." Nothing moves. A spill cell has no formula of its own, so a rewrite would turn it into a literal. |
| `s`/`S` in Range Select started from CellEdit (`Ctrl-r`) | Do nothing. Leaving that mode pastes the range into the cell being edited. |

## Comparison value

Sort compares the computed value of each key cell, not its formula text. A
formula cell that shows `42` sorts as the number 42.

## Technical approach

ironcalc (0.8.3, and the latest code as of 2026-10-23) has no action that
sorts data in place. Its `SORT`/`SORTBY` formula functions write a sorted
copy elsewhere, and its Excel-order comparison function is private. So
sheetui sorts by rewriting the cells of the sort area in the new order (D7):

1. Check the sort area. If it overlaps a merged-cell region, stop with an
   error.
2. Read the computed value of every key cell for the rows to sort.
3. Compute the sorted order (stable), comparing key by key.
4. For each target row, read every cell of its source row, from column A to
   the last used column:
   - the content, as ironcalc `Model::extend_to(source → target)` gives it.
     For a formula this is the formula translated as a copy does
     (`=B2*2` from row 2 becomes `=B5*2` in row 5). For a value it is the
     value text.
   - the full cell style (`UserModel::get_cell_style`).
5. Pause evaluation. Write every content to its target cell. A value that
   would read as a formula is written as text, the same way the CSV loader
   does (`update_cell_literal`). Write all styles with one
   `UserModel::on_paste_styles` call over the sort area.
6. Resume evaluation and evaluate one time.

Read everything before writing anything, so no read sees a half-sorted sheet.

### What moves and what stays

| Part | Behavior |
|------|----------|
| Cell values | Move with the row. |
| Cell formulas | Move with the row, translated as a copy. |
| Cell style (fill, font, bold, italic, borders, alignment, number format) | Moves with the row. |
| Row height and row-level style | Stay with the row position. They do not move. |
| Formulas outside the sort area that point into it | Keep their old address, so they now see different data. This matches Excel. |
| Formulas in a sorted row that point at another row (a running total, `=C1+B2`) | Shift with the row as in a copy, so the results change. This matches Excel. |
| Merged cells | A sort area that overlaps one is refused with an error. This matches Excel. |
| Comments, conditional-formatting ranges | Stay at their positions. sheetui cannot create these; they come only from imported `.xlsx` files. |

### Performance (measured, release build)

| Rows × columns | Time |
|----------------|------|
| 1,000 × 12 | 36 ms |
| 10,000 × 12 | 312 ms |

### Alternatives considered

- **ironcalc `move_rows_action`, one move for each row.** Every formula in
  the book that points at a moved row follows it. Rejected: each move shifts
  all rows between source and target and scans every formula. Measured:
  about 8 s for 1,000 rows and about 16 min for 10,000 rows.
- **Row moves with a row limit.** Rejected: large CSV files could not be
  sorted.
- **Write a sort action for ironcalc and contribute it.** Possible later.
  sheetui would wait for a release.
- **Rewrite values only, with no formula translation.** Rejected: `=B2*2`
  would still point at row 2 after the move.

### Consequence for column scope

The sort rewrites whole rows. A range selection limits which rows sort, but
not which columns. Cells to the right or left of the selection move with
their rows (D2).

## Decisions

- **D1 — How the user starts a sort: command and range key.**
  - `:sort` from Navigate mode, in the command form above.
  - In Range Select mode, `s` sorts descending (the default) and `S` sorts
    ascending. The rows are those of the range from the range start to the
    cursor. The key column is the cursor's column. Range Select mode then
    ends. If no range start is set, or Range Select was started from
    CellEdit, the keys do nothing.
- **D2 — Column scope: whole rows move.** Every cell of a sorted row moves,
  from column A to the last used column. A range selection limits the rows,
  not the columns. The help text must say this.
- **D3 — Header rows: `skip <N>`.** Headers can be more than one row, so the
  command takes a count. `:sort skip N ...` keeps the first N rows of the sort
  area in place, with or without a range selection. The word is `skip`, not
  `header`. `skip` cannot be a column name (columns are at most 3 letters).
  Rejected: a bare leading number (`:sort 2 B` reads as "start at row 2"),
  auto-detecting the header (wrong on all-text tables), and frozen rows as the
  header signal (hidden, and sheetui does not show frozen rows). The `s`/`S`
  keys in Range Select mode do not skip rows; the selection sets the rows.
- **D4 — Undo: later.** Sort ships without undo. To recover from a wrong sort,
  reload the file. The help text must warn about this. When undo comes, a sort
  must undo as one step. (ironcalc records each cell write as its own undo
  step, so this will need work then.)
- **D5 — Multiple key columns: now.** `:sort` takes one or more key columns,
  each with its own direction written as a word after it (`:sort B asc C`).
  We do this now so the command form does not change after people use it.
  The `s`/`S` keys in Range Select mode stay single-key (the cursor's column).
- **D6 — Default direction: descending.** A key with no direction sorts
  descending, so `:sort` puts the largest value of the cursor's column at the
  top. `asc` must be written. In Range Select mode the plain key follows the
  default: `s` is descending, `S` is ascending. Blank cells still go to the
  bottom in both directions.
- **D7 — Technical approach: rewrite cells in sorted order.** Replaces the
  row-move approach, which was too slow (measured above). Cell values,
  formulas (translated as a copy), and cell styles move. Row height and
  row-level style do not move. Formula behavior follows Excel's sort rules.
  A sort area with merged cells is refused.

## Docs to update

- `docs/command.md` — the `:sort` command.
- `docs/visual.md` — the `s` and `S` keys.

The help screens load these files with `include_str!`, so there is no other
copy to update.
