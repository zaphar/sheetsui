//! Command mode command parsers.
use slice_cursor::{Cursor, Seekable, Span, SpanRange, StrCursor};

#[derive(Debug, PartialEq, Eq)]
pub enum Cmd<'a> {
    Write(Option<&'a str>),
    InsertRow(usize),
    InsertColumns(usize),
    Edit(&'a str),
    Help(Option<&'a str>),
}

pub fn parse<'cmd, 'i: 'cmd>(input: &'i str) -> Result<Option<Cmd<'cmd>>, &'static str> {
    let cursor = StrCursor::new(input);
    // try consume write command.
    if let Some(cmd) = try_consume_write(cursor.clone()) {
        return Ok(Some(cmd));
    }
    // try consume insert-row command.
    if let Some(cmd) = try_consume_insert_row(cursor.clone())? {
        return Ok(Some(cmd));
    }
    // try consume insert-col command.
    if let Some(cmd) = try_consume_insert_column(cursor.clone()) {
        return Ok(Some(cmd));
    }
    // try consume edit command.
    if let Some(cmd) = try_consume_edit(cursor.clone()) {
        return Ok(Some(cmd));
    }
    // try consume help command.
    if let Some(cmd) = try_consume_help(cursor.clone()) {
        return Ok(Some(cmd));
    }
    Ok(None)
}

const WRITE: &'static str = "write";

pub fn try_consume_write<'cmd, 'i: 'cmd>(mut input: StrCursor<'i>) -> Option<Cmd<'cmd>> {
    let prefix_len = WRITE.len();
    let full_length = dbg!(input.span(..).len());
    let arg = if full_length >= prefix_len && input.span(..prefix_len) == WRITE {
        input.seek(prefix_len);
        // Should we check for whitespace?
        input.span(prefix_len..)
    } else if full_length >= 2 && input.span(..2) == "w " {
        input.span(2..)
        // Should we check for whitespace?
    } else {
        return None;
    }
    .trim();
    return Some(Cmd::Write(if arg.is_empty() { None } else { Some(arg) }));
}

const IR: &'static str = "ir";
const INSERT_ROW: &'static str = "insert-row";

pub fn try_consume_insert_row<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    let prefix_len = INSERT_ROW.len();
    let second_prefix_len = IR.len();
    let full_length = input.span(..).len();
    let arg =
    if full_length >= prefix_len && input.span(..prefix_len) == INSERT_ROW {
        input.seek(prefix_len);
        // Should we check for whitespace?
        input.span(prefix_len..)
    } else if full_length >= second_prefix_len && input.span(..second_prefix_len) == IR {
        input.span(second_prefix_len..)
        // Should we check for whitespace?
    } else {
        return Ok(None);
    }
    .trim();
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

pub fn try_consume_insert_column<'cmd, 'i: 'cmd>(mut input: StrCursor<'i>) -> Option<Cmd<'cmd>> {
    todo!("insert-column not yet implemented")
}

pub fn try_consume_edit<'cmd, 'i: 'cmd>(mut input: StrCursor<'i>) -> Option<Cmd<'cmd>> {
    todo!("edit not yet implemented")
}

pub fn try_consume_help<'cmd, 'i: 'cmd>(mut input: StrCursor<'i>) -> Option<Cmd<'cmd>> {
    todo!("help not yet implemented")
}
