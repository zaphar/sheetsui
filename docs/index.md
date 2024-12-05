# The sheetui user documentation

## Running sheetui

`sheetui --help` will print out help for the command line tags.

Currently this will print out:

```sh
Usage: sheetui [OPTIONS] <WORKBOOK>

Arguments:
  <WORKBOOK>

Options:
  -l, --locale-name <LOCALE_NAME>      [default: en]
  -t, --timezone-name <TIMEZONE_NAME>  [default: America/New_York]
      --log-input <LOG_INPUT>
  -h, --help                           Print help
  -V, --version                        Print version
```

## User Interface

The sheetui user interface is loosely inspired by vim. It is a modal interface that is entirely keyboard driven. At nearly any time you can type `Alt-h` to get some context sensitive help.

### Navigation Mode

The interface will start out in navigation mode. You can navigate around the table and between the sheets using the following keybinds:

**Cell Navigation**

* `h`, ⬆️, and `TAB` will move one cell to the left.
* `l` and, ➡️ will move one cell to the right.
* `j`, ⬇️, and `Enter` will move one cell down.
* `k` ⬆️, will move one cell up.

**Sheet Navigation**

* `Ctrl-n` moves to the next sheet
* `Ctrl-p` moves to the prev sheet

Sheet navigation moving will loop around when you reach the ends.

**Numeric prefixes**

You can prefix each of the keybinds above with a numeric prefix to do them that many times. So typing `123h` will move to the left 123 times. Hitting `Esc` will clear the numeric prefix if you want to cancel it.

**Modifying the Sheet or Cells**

* `e` or `i` will enter CellEdit mode for the current cell.
* `Ctrl-h` will shorten the width of the column you are on.
* `Ctrl-l` will lengthen the width of the column you are on.

**Other Keybindings**

* `Ctrl-s` will save the sheet.
* `q` will exit the application.
* `:` will enter CommandMode.
 
<aside>Note that for `q` this will not currently prompt you if the sheet is not saved.</aside>

### CellEdit Mode

You enter CellEdit mode by hitting `e` or `i` while in navigation mode. Type what you want into the cell.

Starting with:

* `=` will treat what you type as a formula.
* `$` will treat it as us currency.

Typing a number will treat the contents as a number. While typing non-numeric text will treat it as text content. <aside>We do not yet support modifying the type of a cell after the fact. We may add this in the future.</aside>

For the most part this should work the same way you expect a spreadsheet to work.

* `Enter` will update the cell contents.
* `Esc` will cancel editing the cell and leave it unedited.

`Ctrl-r` will enter range select mode when editing a formula. You can navigate around the
sheet and hit space to select that cell in the sheet to set the start of the range. Navigate some more and hit space to set the end of the range.

You can find the functions we support documented here: [ironcalc docs](https://docs.ironcalc.com/functions/lookup-and-reference.html)

### Command Mode

You enter command mode by typing `:` while in navigation mode. You can then type a command and hit `Enter` to execute it or `Esc` to cancel.

The currently supported commands are:

* `write [path]` save the current spreadsheet. If the path is provided it will save it to that path. If omitted it will save to the path you are currently editing. `w` is a shorthand alias for this command.
* `insert-rows [number]` Inserts a row into the sheet at your current row. If the number is provided then inserts that many rows. If omitted then just inserts one.
* `insert-cols [number]` Just line `insert-rows` but for columns.
* `rename-sheet [idx] <name>` rename a sheet. If the idx is provide then renames that sheet. If omitted then it renames the current sheet.
* `new-sheet [name]` Creates a new sheet. If the name is provided then uses that. If omitted then uses a default sheet name.
* `select-sheet <name>` Select a sheet by name.
* `edit <path>` Edit a new spreadsheet at the current path. `e` is a shorthand alias for this command. 
* `quit` Quits the application. `q` is a shorthand alias for this command.

<aside>Note that in the case of `quit` and `edit` that we do not currently prompt you if the current spreadsheet has not been saved yet. So your changes will be discarded if you have not saved first.</aside>

### Range Select Mode

Range Select mode copies a range reference for use later. You can enter range select mode from CellEdit mode with `CTRL-r`.

* `h`, `j`, `k`, `l` will navigate around the sheet.
* `Ctrl-n`, `Ctrl-p` will navigate between sheets.
* ` ` the spacebar will select the start and end of the range respectively.

When you have selected the end of the range you will exit range select mode and the range reference will be placed into the cell contents you are editing.

<aside>We only support continuous ranges for the moment. Planned for discontinuous ranges still needs the interaction interface to be determined.</aside>
