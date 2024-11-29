//! Command mode command parsers.
use slice_utils::{Measured, Peekable, Seekable, Span, StrCursor};

/// A parsed command entered in during command mode.
#[derive(Debug, PartialEq, Eq)]
pub enum Cmd<'a> {
    Write(Option<&'a str>),
    InsertRow(usize),
    InsertColumns(usize),
    RenameSheet(Option<usize>, &'a str),
    NewSheet(Option<&'a str>),
    SelectSheet(&'a str),
    Edit(&'a str),
    Help(Option<&'a str>),
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
    //// try consume insert-col command.
    if let Some(cmd) = try_consume_insert_column(cursor.clone())? {
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
        return Err("Invalid command: Did you mean to type `write <arg>`?");
    }
    let arg = input.span(0..).trim();
    return Ok(Some(Cmd::Write(if arg.is_empty() {
        None
    } else {
        Some(arg)
    })));
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
        return Err("Invalid command: Did you mean to type `write <arg>`?");
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
        return Err("Invalid command: Did you mean to type `write <sheet-name>`?");
    }
    let arg = input.span(0..).trim();
    if arg.is_empty() {
        return Err("Invalid command: Did you forget the sheet name? `write <sheet-name>`?");
    }
    return Ok(Some(Cmd::SelectSheet(arg)));
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
    return Ok(Some(Cmd::InsertRow(if arg.is_empty() {
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
            return Err("You must pass in a non negative number for the row count");
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
        return Err("Invalid command: `rename-sheet` requires a sheet name argument?");
    }
    return Ok(Some(Cmd::RenameSheet(idx, arg)));
}

fn try_consume_usize<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> (Option<usize>, StrCursor<'i>) {
    let mut out = String::new();
    let original_input = input.clone();
    while input.peek_next().map(|c| (*c as char).is_ascii_digit()).unwrap_or(false) {
        out.push(*input.next().unwrap() as char);
    }
    if out.len() > 0 {
        return (Some(out.parse().unwrap()), input.clone());
    }
    (None, original_input)
}
