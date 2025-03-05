//! Command mode command parsers.
use slice_utils::{Measured, Peekable, Seekable, Span, StrCursor};

/// A parsed command entered in during command mode.
#[derive(Debug, PartialEq, Eq)]
pub enum Cmd<'a> {
    Write(Option<&'a str>),
    InsertRows(usize),
    InsertColumns(usize),
    ColorRows(Option<usize>, String),
    ColorColumns(Option<usize>, String),
    ColorCell(String),
    RenameSheet(Option<usize>, &'a str),
    NewSheet(Option<&'a str>),
    SelectSheet(&'a str),
    Edit(&'a str),
    Help(Option<&'a str>),
    Export(&'a str),
    Quit,
}

/// Parse command text into a `Cmd`.
pub fn parse<'cmd, 'i: 'cmd>(input: &'i str) -> Result<Option<Cmd<'cmd>>, &'static str> {
    let cursor = StrCursor::new(input);
    // try consume write command.
    if let Some(cmd) = try_consume_write(cursor.clone())? {
        return Ok(Some(cmd));
    }
    if let Some(cmd) = try_consume_new_sheet(cursor.clone())? {
        return Ok(Some(cmd));
    }
    if let Some(cmd) = try_consume_select_sheet(cursor.clone())? {
        return Ok(Some(cmd));
    }
    // try consume insert-row command.
    if let Some(cmd) = try_consume_insert_row(cursor.clone())? {
        return Ok(Some(cmd));
    }
    // try consume insert-col command.
    if let Some(cmd) = try_consume_insert_column(cursor.clone())? {
        return Ok(Some(cmd));
    }
    // Try consume export
    if let Some(cmd) = try_consume_export(cursor.clone())? {
        return Ok(Some(cmd));
    }
    // try consume edit command.
    if let Some(cmd) = try_consume_edit(cursor.clone())? {
        return Ok(Some(cmd));
    }
    // try consume help command.
    if let Some(cmd) = try_consume_help(cursor.clone())? {
        return Ok(Some(cmd));
    }
    // try consume quit command.
    if let Some(cmd) = try_consume_quit(cursor.clone())? {
        return Ok(Some(cmd));
    }
    if let Some(cmd) = try_consume_rename_sheet(cursor.clone())? {
        return Ok(Some(cmd));
    }
    if let Some(cmd) = try_consume_color_rows(cursor.clone())? {
        return Ok(Some(cmd));
    }
    if let Some(cmd) = try_consume_color_columns(cursor.clone())? {
        return Ok(Some(cmd));
    }
    if let Some(cmd) = try_consume_color_cell(cursor.clone())? {
        return Ok(Some(cmd));
    }
    Ok(None)
}

fn compare<'i>(input: StrCursor<'i>, compare: &str) -> bool {
    input.remaining() >= compare.len() && input.span(0..compare.len()) == compare
}

fn is_ws<'r, 'i: 'r>(input: &'r mut StrCursor<'i>) -> bool {
    match input.peek_next() {
        Some(b) => {
            if *b == (' ' as u8) || *b == ('\t' as u8) || *b == ('\n' as u8) || *b == ('\r' as u8) {
                input.next();
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

fn try_consume_write<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const SHORT: &'static str = "w";
    const LONG: &'static str = "write";

    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else if compare(input.clone(), SHORT) {
        input.seek(SHORT.len());
        // Should we check for whitespace?
    } else {
        return Ok(None);
    }
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err("Invalid command: Did you mean to type `write <path>`?");
    }
    let arg = input.span(0..).trim();
    return Ok(Some(Cmd::Write(if arg.is_empty() {
        None
    } else {
        Some(arg)
    })));
}

fn try_consume_export<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const SHORT: &'static str = "ex";
    const LONG: &'static str = "export";

    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else if compare(input.clone(), SHORT) {
        input.seek(SHORT.len());
        // Should we check for whitespace?
    } else {
        return Ok(None);
    }
    if input.remaining() == 0 || !is_ws(&mut input) {
        return Err("Invalid command: Did you mean to type `export <path>`?");
    }
    let arg = input.span(0..).trim();
    return Ok(Some(Cmd::Export(arg)));
}

fn try_consume_new_sheet<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const LONG: &'static str = "new-sheet";

    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else {
        return Ok(None);
    }
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err("Invalid command: Did you mean to type `new-sheet <arg>`?");
    }
    let arg = input.span(0..).trim();
    return Ok(Some(Cmd::NewSheet(if arg.is_empty() {
        None
    } else {
        Some(arg)
    })));
}

fn try_consume_select_sheet<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const LONG: &'static str = "select-sheet";

    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else {
        return Ok(None);
    }
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err("Invalid command: Did you mean to type `select-sheet <sheet-name>`?");
    }
    let arg = input.span(0..).trim();
    if arg.is_empty() {
        return Err("Invalid command: Did you forget the sheet name? `select-sheet <sheet-name>`?");
    }
    return Ok(Some(Cmd::SelectSheet(arg)));
}

fn try_consume_color_cell<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const SHORT: &'static str = "cc";
    const LONG: &'static str = "color-cell";
    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else if compare(input.clone(), SHORT) {
        input.seek(SHORT.len());
    } else {
        return Ok(None);
    };
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err("Invalid command: Did you mean to type `color-cell <color>`?");
    }
    let arg = parse_color(input.span(0..).trim())?;
    return Ok(Some(Cmd::ColorCell(arg)));
}

fn try_consume_insert_row<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const SHORT: &'static str = "ir";
    const LONG: &'static str = "insert-rows";

    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else if compare(input.clone(), SHORT) {
        input.seek(SHORT.len());
    } else {
        return Ok(None);
    };
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err("Invalid command: Did you mean to type `insert-rows <arg>`?");
    }
    let arg = input.span(0..).trim();
    return Ok(Some(Cmd::InsertRows(if arg.is_empty() {
        1
    } else {
        if let Ok(count) = arg.parse() {
            count
        } else {
            return Err("You must pass in a non negative number for the row count");
        }
    })));
}

fn try_consume_insert_column<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const SHORT: &'static str = "ic";
    const LONG: &'static str = "insert-cols";

    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else if compare(input.clone(), SHORT) {
        input.seek(SHORT.len());
    } else {
        return Ok(None);
    };
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err("Invalid command: Did you mean to type `insert-cols <arg>`?");
    }
    let arg = input.span(0..).trim();
    return Ok(Some(Cmd::InsertColumns(if arg.is_empty() {
        1
    } else {
        if let Ok(count) = arg.parse() {
            count
        } else {
            return Err("You must pass in a non negative number for the column count");
        }
    })));
}

fn try_consume_edit<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const SHORT: &'static str = "e";
    const LONG: &'static str = "edit";

    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else if compare(input.clone(), SHORT) {
        input.seek(SHORT.len());
    } else {
        return Ok(None);
    };
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err("Invalid command: Did you mean to type `edit <arg>`?");
    }
    let arg = input.span(0..).trim();
    return Ok(Some(Cmd::Edit(if arg.is_empty() {
        return Err("You must pass in a path to edit");
    } else {
        arg
    })));
}

fn try_consume_help<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const SHORT: &'static str = "?";
    const LONG: &'static str = "help";

    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else if compare(input.clone(), SHORT) {
        input.seek(SHORT.len());
        // Should we check for whitespace?
    } else {
        return Ok(None);
    }
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err("Invalid command: Did you mean to type `help <arg>`?");
    }
    let arg = input.span(0..).trim();
    return Ok(Some(Cmd::Help(if arg.is_empty() {
        None
    } else {
        Some(arg)
    })));
}

fn try_consume_quit<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const SHORT: &'static str = "q";
    const LONG: &'static str = "quit";

    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else if compare(input.clone(), SHORT) {
        input.seek(SHORT.len());
        // Should we check for whitespace?
    } else {
        return Ok(None);
    }
    if input.remaining() > 0 {
        return Err("Invalid command: Quit does not take an argument");
    }
    return Ok(Some(Cmd::Quit));
}

fn try_consume_rename_sheet<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const LONG: &'static str = "rename-sheet";
    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else {
        return Ok(None);
    }
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err("Invalid command: Did you mean to type `rename-sheet [idx] <new-name>`?");
    }
    let (idx, rest) = try_consume_usize(input.clone());
    let arg = rest.span(0..).trim();
    if arg.is_empty() {
        return Err("Invalid command: `rename-sheet` requires a sheet name argument");
    }
    return Ok(Some(Cmd::RenameSheet(idx, arg)));
}

fn try_consume_color_rows<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const LONG: &'static str = "color-rows";
    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else {
        return Ok(None);
    }
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err("Invalid command: Did you mean to type `color-rows [count] <color>`?");
    }
    let (idx, rest) = try_consume_usize(input.clone());
    let arg = parse_color(rest.span(0..).trim())?;
    return Ok(Some(Cmd::ColorRows(idx, arg)));
}

fn try_consume_color_columns<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const LONG: &'static str = "color-columns";
    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else {
        return Ok(None);
    }
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err("Invalid command: Did you mean to type `color-columns [count] <color>`?");
    }
    let (idx, rest) = try_consume_usize(input.clone());
    let arg = parse_color(rest.span(0..).trim())?;
    return Ok(Some(Cmd::ColorColumns(idx, arg)));
}

pub(crate) fn parse_color(color: &str) -> Result<String, &'static str> {
    use colorsys::{Ansi256, Rgb};
    if color.is_empty() {
        return Err("Invalid command: `color-columns` requires a color argument");
    }
    let parsed = match color.to_lowercase().as_str() {
        "black" => Ansi256::new(0).as_rgb().to_hex_string(),
        "red" => Ansi256::new(1).as_rgb().to_hex_string(),
        "green" => Ansi256::new(2).as_rgb().to_hex_string(),
        "yellow" => Ansi256::new(3).as_rgb().to_hex_string(),
        "blue" => Ansi256::new(4).as_rgb().to_hex_string(),
        "magenta" => Ansi256::new(5).as_rgb().to_hex_string(),
        "cyan" => Ansi256::new(6).as_rgb().to_hex_string(),
        "gray" | "grey" => Ansi256::new(7).as_rgb().to_hex_string(),
        "darkgrey" | "darkgray" => Ansi256::new(8).as_rgb().to_hex_string(),
        "lightred" => Ansi256::new(9).as_rgb().to_hex_string(),
        "lightgreen" => Ansi256::new(10).as_rgb().to_hex_string(),
        "lightyellow" => Ansi256::new(11).as_rgb().to_hex_string(),
        "lightblue" => Ansi256::new(12).as_rgb().to_hex_string(),
        "lightmagenta" => Ansi256::new(13).as_rgb().to_hex_string(),
        "lightcyan" => Ansi256::new(14).as_rgb().to_hex_string(),
        "white" => Ansi256::new(15).as_rgb().to_hex_string(),
        candidate => {
            if candidate.starts_with("#") {
                candidate.to_string()
            } else if candidate.starts_with("rgb(") {
                if let Ok(rgb) = <Rgb as std::str::FromStr>::from_str(candidate) {
                    // Note that the colorsys rgb model clamps the f64 values to no more
                    // than 255.0 so the below casts are safe.
                    rgb.to_hex_string()
                } else {
                    return Err("Invalid color");
                }
            } else {
                return Err("Invalid color");
            }
        }
    };
    Ok(parsed)
}

fn try_consume_usize<'cmd, 'i: 'cmd>(mut input: StrCursor<'i>) -> (Option<usize>, StrCursor<'i>) {
    let mut out = String::new();
    let original_input = input.clone();
    while input
        .peek_next()
        .map(|c| (*c as char).is_ascii_digit())
        .unwrap_or(false)
    {
        out.push(*input.next().unwrap() as char);
    }
    if out.len() > 0 {
        return (Some(out.parse().unwrap()), input.clone());
    }
    (None, original_input)
}
