# Navigation Mode

The interface will start out in navigation mode. You can navigate around the
table and between the sheets using the following keybinds:

## Cell Navigation

* `h`, ⬆️, and `TAB` will move one cell to the left.
* `l` and, ➡️ will move one cell to the right.
* `j`, ⬇️, and `Enter` will move one cell down.
* `k` ⬆️, will move one cell up.
* `d` will delete the contents of the selected cell leaving style untouched
* `D` will delete the contents of the selected cell including any style
* `gg` will go to the top row in the current column

## Sheet Navigation

* `Ctrl-n` moves to the next sheet
* `Ctrl-p` moves to the prev sheet

Sheet navigation moving will loop around when you reach the ends.

## Numeric prefixes

You can prefix each of the keybinds above with a numeric prefix to do them that
many times. So typing `123h` will move to the left 123 times. Hitting `Esc`
will clear the numeric prefix if you want to cancel it.

**Modifying the Sheet or Cells**

* `e` or `i` will enter CellEdit mode for the current cell.
* 'I' will toggle italic on the cell. 'B' will toggle bold.
* `Ctrl-h` will shorten the width of the column you are on.
* `Ctrl-l` will lengthen the width of the column you are on.

## Other Keybindings

* `Ctrl-r` will enter range selection mode.
* `v` will enter range selection mode with the start of the range already selected.
* `Ctrl-s` will save the sheet.
* `Ctrl-c`, `y` Copy the cell or range contents.
* `Ctrl-v`, `p` Paste into the sheet.
* `Ctrl-Shift-C` Copy the cell or range formatted content.
* `q` will exit the application.
* `:` will enter CommandMode.

Range selections made from navigation mode will be available to paste into a Cell Edit.

<aside>Note that for `q` this will not currently prompt you if the sheet is not
saved.</aside>

