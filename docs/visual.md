# Range Select Mode

Range Select mode copies a range reference for use later or delete a range's contents. You can enter range
select mode from CellEdit mode with `CTRL-r`.

* `h`, `j`, `k`, `l` will navigate around the sheet.
* `Ctrl-n`, `Ctrl-p` will navigate between sheets.
* `Ctrl-c`, `y` Copy the cell or range formatted contents.
* `Ctrl-Shift-C`, 'Y' Copy the cell or range content.
* `The spacebar will select the start and end of the range respectively.
* `d` will delete the contents of the range leaving any style untouched
* `D` will delete the contents of the range including any style

When you have selected the end of the range you will exit range select mode and
the range reference will be placed into the cell contents you are editing.

<aside>We only support continuous ranges for the moment. Planned for
discontinuous ranges still needs the interaction interface to be
determined.</aside>
