# Command Mode

You enter command mode by typing `:` while in navigation mode. You can then
type a command and hit `Enter` to execute it or `Esc` to cancel.

The currently supported commands are:

* `write [path]` save the current spreadsheet. If the path is provided it will save it to that path. If omitted it will save to the path you are currently editing. `w` is a shorthand alias for this command.
* `insert-rows [number]` Inserts a row into the sheet at your current row. If the number is provided then inserts that many rows. If omitted then just inserts one.
* `insert-cols [number]` Just line `insert-rows` but for columns.
* `rename-sheet [idx] <name>` rename a sheet. If the idx is provide then renames that sheet. If omitted then it renames the current sheet.
* `new-sheet [name]` Creates a new sheet. If the name is provided then uses that. If omitted then uses a default sheet name.
* `select-sheet <name>` Select a sheet by name.
* `edit <path>` Edit a new spreadsheet at the current path. `e` is a shorthand alias for this command. 
* `quit` Quits the application. `q` is a shorthand alias for this command.

<aside>Note that in the case of `quit` and `edit` that we do not currently
prompt you if the current spreadsheet has not been saved yet. So your changes
will be discarded if you have not saved first.</aside>

