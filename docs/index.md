# The sheetui user documentation

## Running sheetui

`sheetui --help` will print out help for the command line tags.

Currently this will print out:

```sh
Usage: sheetui [OPTIONS] [WORKBOOK]

Arguments:
  [WORKBOOK]

Options:
  -l, --locale-name <LOCALE_NAME>      [default: en]
  -t, --timezone-name <TIMEZONE_NAME>  [default: America/New_York]
      --log-input <LOG_INPUT>
  -h, --help                           Print help
  -V, --version                        Print version
```

If you do not provide a workbook path, sheetui will open an empty workbook.

## Supported formats

sheetui supports two spreadsheet file formats:

* **`.sui`** — sheetui's native plain-text format. Human-readable, diff-friendly,
  and the default for new workbooks. See [File Formats](./file-formats.md) for
  the complete format reference.
* **`.xlsx`** — Excel-compatible format, backed by the
  [ironcalc](https://docs.ironcalc.com/) library. Use this when sharing files
  with other spreadsheet applications.

When you open or save a file, sheetui detects the format from the file
extension. If you start sheetui without a workbook path, an empty workbook is
created and will be saved as `Untitled.sui` in the current directory when you
first write it.

CSV export is available via the `export-csv` command (see
[Command Mode](./command.md)).

## User Interface

The sheetui user interface is loosely inspired by vim. It is a modal interface
that is entirely keyboard driven. At nearly any time you can type `Alt-h` to
get some context sensitive help.

### Modal Docs

* [Navigation](./navigation.md)
* [Edit](./edit.md)
* [Visual](./visual.md)
* [Command](./command.md)
