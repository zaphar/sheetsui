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

## Supported formats

Currently we only support the [ironcalc](https://docs.ironcalc.com/) xlsx
features for spreadsheet. I plan to handle csv import and export at some point.
I also might support other export formats as well but for the moment just csv
and it's variants such as tsv are in the roadmap.

## User Interface

The sheetui user interface is loosely inspired by vim. It is a modal interface
that is entirely keyboard driven. At nearly any time you can type `Alt-h` to
get some context sensitive help.

### Modal Docs

* [Navigation](./navigation.md)
* [Edit](./edit.md)
* [Visual](./visual.md)
* [Command](./command.md)
