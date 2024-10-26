//! Parser for a spreadsheet formula.

use a1::A1;

use std::iter::Iterator;
use std::ops::{RangeBounds, Index};
use std::slice::SliceIndex;

/// A segment of a Formula. A formula is segmented into either
/// [Placeholder](FormulaSegment::Placeholder) address lookups or partial
/// formula text.
#[derive(Debug)]
pub enum FormulaSegment<'source> {
    Placeholder(A1),
    Unparsed(&'source str),
}

/// A Parsed Formula AST
#[derive(Debug)]
pub struct PreParsed<'source> {
    segments: Vec<FormulaSegment<'source>>,
}

#[derive(thiserror::Error, Debug)]
pub enum FormulaError {
    #[error("Failed to Parse formula")]
    ParseFailure,
}

impl<'source> PreParsed<'source> {
    pub fn is_formula(candidate: &str) -> bool {
        candidate.len() > 0 && candidate.bytes().nth(0).unwrap() == b'='
    }

    pub fn try_parse(candidate: &'source str) -> Result<Option<Self>, FormulaError> {
        if !Self::is_formula(candidate) {
            return Ok(None);
        }
        // TODO(zaphar) Gather up the references for this Formula
        Ok(Some(PreParsed {
            segments: parse_segments(candidate.into()),
        }))
    }
}

fn parse_segments<'source>(mut iter: StrIter<'source>) -> Vec<FormulaSegment<'source>> {
    let mut segments = Vec::new();
    let mut buffer = Vec::new();
    loop {
        if let Some((addr, i)) = try_parse_addr(iter.clone()) {
            segments.push(FormulaSegment::Placeholder(addr));
            buffer.clear();
            iter = i;
        } else {
            if let Some(b) = iter.peek_next() {
                buffer.push(b);
            }
        }
        if let None = iter.next() {
            break;
        }
    }
    return segments;
}

pub fn try_parse_addr<'source>(iter: StrIter<'source>) -> Option<(a1::A1, StrIter<'source>)> {
    let start = iter.clone();
    if let Ok(addr) = a1::new(start.rest()) {
        return Some((addr, start));
    }
    // Consume 1 capital
    //if let Some(i) = consume_capital(iter.clone()) {
    //    iter = i;
    //} else {
    //    // This isn't a capitable letter
    //    return None
    //}
    //// maybe Consume 2 capital letters
    //if let Some(i) = consume_capital(iter.clone()) {
    //    iter = i;
    //}
    //if let Some(b':') = iter.peek_next() {
    //    iter.next();
    //}
    //// Consume 1 capital
    //if let Some(i) = consume_capital(iter.clone()) {
    //    iter = i;
    //} else {
    //    // This isn't a capitable letter
    //    return None
    //}
    //// maybe Consume 2 capital letters
    //if let Some(i) = consume_capital(iter.clone()) {
    //    iter = i;
    //}
    return None;
}

fn consume_capital<'source>(mut iter: StrIter<'source>) -> Option<StrIter<'source>> {
    if let Some(c) = iter.peek_next() {
        match *c {
            b'A' | b'B' | b'C' | b'D' | b'E' | b'F' | b'G' | b'H' | b'I' | b'J' | b'K'
            | b'L' | b'M' | b'N' | b'O' | b'P' | b'Q' | b'R' | b'S' | b'T' | b'U'
            | b'V' | b'W'  | b'X' | b'Y' | b'Z' => {
                iter.next();
                return Some(iter);
            }
            _ => {
                return None;
            }
        }
    }
    return None;
}

fn consume_ws<'source>(mut iter: StrIter<'source>) -> Option<StrIter<'source>> {
    if let Some(c) = iter.peek_next() {
        match *c {
            b' ' | b'\t' | b'\r' | b'\n' => {
                iter.next();
                return Some(iter);
            }
            _ => {
                return None;
            }
        }
    }
    return None;
}

/// Implements `InputIter` for any slice of T.
#[derive(Debug)]
pub struct StrIter<'a> {
    source: &'a str,
    pub offset: usize,
}

impl<'a> StrIter<'a> {
    /// new constructs a StrIter from a Slice of T.
    pub fn new(source: &'a str) -> Self {
        StrIter { source, offset: 0 }
    }

    fn seek(&mut self, to: usize) -> usize {
        let self_len = self.source.len();
        let offset = if self_len > to { to } else { self_len };
        self.offset = offset;
        self.offset
    }

    fn peek_next(&self) -> Option<&'a u8> {
        self.source.as_bytes().get(self.offset)
    }

    fn get_range<R: RangeBounds<usize> + SliceIndex<str, Output=str>>(&self, range: R) -> &'a str {
        &self.source[range]
    }

    pub fn rest(&'a self) -> &'a str {
        &self[self.offset..]
    }
}

impl<'a> Iterator for StrIter<'a> {
    type Item = &'a u8;

    fn next(&mut self) -> Option<Self::Item> {
        match self.source.as_bytes().get(self.offset) {
            // TODO count lines and columns.
            Some(item) => {
                self.offset += 1;
                Some(item)
            }
            None => None,
        }
    }
}

impl<'a> Clone for StrIter<'a> {
    fn clone(&self) -> Self {
        StrIter {
            source: self.source,
            offset: self.offset,
        }
    }
}

impl<'a> From<&'a str> for StrIter<'a> {
    fn from(source: &'a str) -> Self {
        Self::new(source)
    }
}

impl<'a, Idx> Index<Idx> for StrIter<'a>
where Idx: RangeBounds<usize> + SliceIndex<str, Output=str>
{
        type Output = Idx::Output;

        fn index(&self, index: Idx) -> &'a Self::Output {
            &self.source[index]
    }
}
