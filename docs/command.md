# Command Mode

You enter command mode by typing `:` while in navigation mode. You can then
type a command and hit `Enter` to execute it or `Esc` to cancel.

The currently supported commands are:

* `write [path]` save the current spreadsheet. If the path is provided it will save it to that path and update the current file path for subsequent saves. If omitted it will save to the path you are currently editing (or `Untitled.sui` for a new empty workbook). The format is auto-detected from the file extension: `.xlsx` uses Excel format, `.csv` writes the first sheet's displayed values, everything else uses the native `.sui` format. `w` is a shorthand alias for this command.
* `insert-rows [number]` Inserts a row into the sheet at your current row. If the number is provided then inserts that many rows. If omitted then just inserts one.
* `insert-cols [number]` Just line `insert-rows` but for columns.
* `color-rows [count] <color>` color rows. The count of rows if given specifies how many rows going down to color. 
* `color-cols [count] <color>` color columns. The count of rows if given specifies how many columns going right to color.
* `color-cell <color>` Color the currently selected cells.
* `rename-sheet [idx] <name>` rename a sheet. If the idx is provide then renames that sheet. If omitted then it renames the current sheet.
* `new-sheet [name]` Creates a new sheet. If the name is provided then uses that. If omitted then uses a default sheet name.
* `select-sheet <name>` Select a sheet by name.
* `edit <path>` Open a spreadsheet from the given path. Format is auto-detected from the file extension (`.xlsx` → Excel, `.csv` → CSV, everything else → `.sui`). `e` is a shorthand alias for this command.
* `help [topic]` Display help for a given topic.
* `export-csv <path>` Export the current sheet to a csv file at `<path>`.
* `quit` Quits the application. `q` is a shorthand alias for this command.
* `system-paste` Paste from the system clipboard
* `sort [skip <N>] [<column> [asc|desc] ...]` Sort rows. See Sorting below.

## Sorting

`sort` sorts the rows of the last range selection. If there is no range
selection, it sorts all the rows of the sheet that have data.

* `sort` sorts by the column the cursor is on, descending.
* `sort asc` sorts by the column the cursor is on, ascending.
* `sort B` sorts by column B, descending.
* `sort B asc C` sorts by column B ascending. Rows with the same value in B
  then sort by column C, descending.
* `sort skip 1 B` keeps the first row in place (for example a header row) and
  sorts the rest. `skip 3` keeps the first 3 rows.

A direction applies only to the column just before it. `sort B C asc` sorts B
descending and C ascending. A column with no direction sorts descending.
`skip` must come first.

Every cell of a sorted row moves: values, formulas, and cell styles. A range
selection limits which rows sort, not which columns. Row height does not move.

The sort order is: numbers, then text (case is ignored), then `TRUE`/`FALSE`,
then errors. Empty cells are always last. Rows with equal values keep their
order. Formula cells sort by their result.

Formulas follow the same rules as in Excel:

* A formula in a sorted row changes as if you copied it to its new row. So
  `=B2*2` that moves to row 5 becomes `=B5*2`.
* A formula outside the sorted rows that points into them keeps its address.
  After the sort it can show a different value.

You cannot sort rows that contain merged cells or array formulas.

<aside>There is no undo yet. If a sort is wrong, reload the file without
saving.</aside>

<aside>Note that in the case of `quit` and `edit` that we do not currently
prompt you if the current spreadsheet has not been saved yet. So your changes
will be discarded if you have not saved first.</aside>

