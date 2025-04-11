//! Ui rendering logic
use std::{path::PathBuf, process::ExitCode, str::FromStr};

use crate::book::{self, AddressRange, Book};

use anyhow::{anyhow, Result};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ironcalc::base::{expressions::types::Area, Model};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout},
    style::{Modifier, Style},
    widgets::Block,
};
use tui_prompts::{State, Status, TextPrompt, TextState};
use tui_textarea::{CursorMove, TextArea};

mod cmd;
mod help;
pub mod render;
#[cfg(test)]
mod test;

use cmd::Cmd;
use render::{markdown::Markdown, viewport::ViewportState};

#[derive(Default, Debug, PartialEq, Clone)]
pub enum Modality {
    #[default]
    Navigate,
    CellEdit,
    Command,
    Dialog,
    RangeSelect,
    Quit,
}

#[derive(Debug, Default)]
pub struct RangeSelection {
    pub original_location: Option<Address>,
    pub start: Option<Address>,
    pub end: Option<Address>,
}

impl RangeSelection {
    pub fn get_range(&self) -> Option<(Address, Address)> {
        if let (Some(start), Some(end)) = (&self.start, &self.end) {
            return Some((
                Address {
                    sheet: start.sheet,
                    row: std::cmp::min(start.row, end.row),
                    col: std::cmp::min(start.col, end.col),
                },
                Address {
                    sheet: end.sheet,
                    row: std::cmp::max(start.row, end.row),
                    col: std::cmp::max(start.col, end.col),
                },
            ));
        }
        None
    }

    pub fn reset_range_selection(&mut self) {
        self.start = None;
        self.end = None;
    }
}

#[derive(Debug)]
pub enum ClipboardContents {
    Cell(String),
    Range(Vec<Vec<String>>),
}

#[derive(Debug)]
pub struct AppState<'ws> {
    pub modality_stack: Vec<Modality>,
    pub viewport_state: ViewportState,
    pub command_state: TextState<'ws>,
    pub numeric_prefix: Vec<char>,
    pub char_queue: Vec<char>,
    pub range_select: RangeSelection,
    pub dialog_scroll: u16,
    dirty: bool,
    popup: Option<Markdown>,
    clipboard: Option<ClipboardContents>,
}

impl<'ws> Default for AppState<'ws> {
    fn default() -> Self {
        AppState {
            modality_stack: vec![Modality::default()],
            viewport_state: Default::default(),
            command_state: Default::default(),
            numeric_prefix: Default::default(),
            char_queue: Default::default(),
            range_select: Default::default(),
            dialog_scroll: 0,
            dirty: false,
            popup: Default::default(),
            clipboard: Default::default(),
        }
    }
}
impl<'ws> AppState<'ws> {
    pub fn modality(&'ws self) -> &'ws Modality {
        self.modality_stack.last().unwrap()
    }

    pub fn pop_modality(&mut self) {
        if self.modality_stack.len() > 1 {
            self.modality_stack.pop();
        }
    }

    pub fn get_n_prefix(&self) -> usize {
        let prefix = self
            .numeric_prefix
            .iter()
            .map(|c| c.to_digit(10).unwrap())
            .fold(Some(0 as usize), |acc, n| {
                acc?.checked_mul(10)?.checked_add(n as usize)
            })
            .unwrap_or(1);
        if prefix == 0 {
            return 1;
        }
        prefix
    }

    pub fn reset_n_prefix(&mut self) {
        self.numeric_prefix.clear();
    }
}

// TODO(jwall): Should we just be using `Area` for this?.
/// The Address in a Table.
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone)]
pub struct Address {
    pub sheet: u32,
    pub row: usize,
    pub col: usize,
}

impl Address {
    pub fn new(row: usize, col: usize) -> Self {
        Self { sheet: 0, row, col }
    }

    pub fn to_range_part(&self) -> String {
        let count = if self.col == 26 {
            1
        } else {
            (self.col / 26) + 1
        };
        format!(
            "{}{}",
            render::viewport::COLNAMES[(self.col - 1) % 26].repeat(count),
            self.row
        )
    }
}

impl Default for Address {
    fn default() -> Self {
        Address::new(1, 1)
    }
}

/// A workspace defining our UI state.
pub struct Workspace<'ws> {
    name: PathBuf,
    book: Book,
    pub(crate) state: AppState<'ws>,
    text_area: TextArea<'ws>,
}

impl<'ws> Workspace<'ws> {
    /// Constructs a new Workspace from an `Book` with a path for the name.
    pub fn new(book: Book, name: PathBuf) -> Self {
        let mut ws = Self {
            book,
            name,
            state: AppState::default(),
            text_area: reset_text_area("".to_owned()),
        };
        ws.handle_movement_change();
        ws
    }

    pub fn new_empty(locale: &str, tz: &str) -> Result<Self> {
        Ok(Self::new(
            Book::from_model(Model::new_empty("", locale, tz).map_err(|e| anyhow!("{}", e))?),
            PathBuf::from_str("Untitled.xlsx").unwrap(),
        ))
    }

    /// Loads a workspace from a path.
    pub fn load(path: &PathBuf, locale: &str, tz: &str) -> Result<Self> {
        let book = load_book(path, locale, tz)?;
        Ok(Workspace::new(book, path.clone()))
    }

    /// Loads a new `Book` into a `Workspace` from a path.
    pub fn load_into<P: Into<PathBuf>>(&mut self, path: P) -> Result<()> {
        let path: PathBuf = path.into();
        // FIXME(zaphar): This should be managed better.
        let book = load_book(&path, "en", "America/New_York")?;
        self.book = book;
        self.name = path;
        Ok(())
    }

    pub fn selected_range_to_string(&self) -> String {
        let state = &self.state;
        if let Some((start, end)) = state.range_select.get_range() {
            let a1 = format!(
                "{}{}",
                start.to_range_part(),
                format!(":{}", end.to_range_part())
            );
            if let Some(ref start_addr) = state.range_select.start {
                if start_addr.sheet != self.book.location.sheet {
                    return format!(
                        "{}!{}",
                        self.book
                            .get_sheet_name_by_idx(start_addr.sheet as usize)
                            .expect("No such sheet index"),
                        a1
                    );
                }
            }
            return a1;
        }
        return String::new();
    }

    /// Move a row down in the current sheet.
    pub fn move_down(&mut self) -> Result<()> {
        let mut loc = self.book.location.clone();
        if loc.row < (book::LAST_ROW as usize) {
            loc.row += 1;
            self.book.move_to(&loc)?;
        }
        Ok(())
    }

    /// Move to the top row without changing columns
    pub fn move_to_top(&mut self) -> Result<()> {
        self.book.move_to(&Address {
            sheet: self.book.location.sheet,
            row: 1,
            col: self.book.location.col,
        })?;
        Ok(())
    }

    /// Move a row up in the current sheet.
    pub fn move_up(&mut self) -> Result<()> {
        let mut loc = self.book.location.clone();
        if loc.row > 1 {
            loc.row -= 1;
            self.book.move_to(&loc)?;
        }
        Ok(())
    }

    /// Move a column to the left in the current sheet.
    pub fn move_left(&mut self) -> Result<()> {
        let mut loc = self.book.location.clone();
        if loc.col > 1 {
            loc.col -= 1;
            self.book.move_to(&loc)?;
        }
        Ok(())
    }

    /// Move a column to the left in the current sheet.
    pub fn move_right(&mut self) -> Result<()> {
        let mut loc = self.book.location.clone();
        if loc.col < (book::LAST_COLUMN as usize) {
            loc.col += 1;
            self.book.move_to(&loc)?;
        }
        Ok(())
    }

    /// Handle input in our ui loop.
    pub fn handle_input(&mut self, evt: Event) -> Result<Option<ExitCode>> {
        if let Event::Key(key) = evt {
            let result = match self.state.modality() {
                Modality::Navigate => self.handle_navigation_input(key)?,
                Modality::CellEdit => self.handle_edit_input(key)?,
                Modality::Command => self.handle_command_input(key)?,
                Modality::Dialog => self.handle_dialog_input(key)?,
                Modality::RangeSelect => self.handle_range_select_input(key)?,
                Modality::Quit => self.handle_quit_dialog(key)?,
            };
            return Ok(result);
        }
        Ok(None)
    }

    fn render_help_text(&self) -> Markdown {
        // TODO(zaphar): We should be sourcing these from our actual help documentation.
        // Ideally we would also render the markdown content properly.
        // https://github.com/zaphar/sheetsui/issues/22
        match self.state.modality() {
            Modality::Navigate => help::to_widget("navigate"),
            Modality::CellEdit => help::to_widget("edit"),
            Modality::Command => help::to_widget("command"),
            Modality::RangeSelect => help::to_widget("visual"),
            _ => help::to_widget(""),
        }
    }

    fn handle_command_input(&mut self, key: event::KeyEvent) -> Result<Option<ExitCode>> {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => return self.exit_command_mode(),
                KeyCode::Char('h') if key.modifiers == KeyModifiers::ALT => {
                    self.enter_dialog_mode(self.render_help_text());
                    return Ok(None);
                }
                _ => {
                    // NOOP
                }
            }
        }
        self.state.command_state.handle_key_event(key);
        Ok(None)
    }

    fn handle_quit_dialog(&mut self, key: event::KeyEvent) -> Result<Option<ExitCode>> {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                    self.exit_quit_mode()?;
                    return Ok(Some(ExitCode::SUCCESS));
                }
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    // We have been asked to save the file first.
                    self.save_file()?;
                    self.exit_quit_mode()?;
                    return Ok(Some(ExitCode::SUCCESS));
                }
                _ => return Ok(None),
            }
        }
        Ok(None)
    }

    fn handle_dialog_input(&mut self, key: event::KeyEvent) -> Result<Option<ExitCode>> {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => self.exit_dialog_mode()?,
                KeyCode::Char('h') if key.modifiers == KeyModifiers::ALT => {
                    self.exit_dialog_mode()?
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    self.state.dialog_scroll = self.state.dialog_scroll.saturating_add(1);
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.state.dialog_scroll = self.state.dialog_scroll.saturating_sub(1);
                }
                code => {
                    if let Some(widget) = &self.state.popup {
                        widget.handle_input(code);
                    }
                }
            }
        }
        Ok(None)
    }

    fn handle_edit_input(&mut self, key: event::KeyEvent) -> Result<Option<ExitCode>> {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Char('h') if key.modifiers == KeyModifiers::ALT => {
                    self.enter_dialog_mode(self.render_help_text());
                    return Ok(None);
                }
                KeyCode::Char('r') if key.modifiers == KeyModifiers::CONTROL => {
                    self.enter_range_select_mode(false);
                    return Ok(None);
                }
                KeyCode::Char('p') if key.modifiers == KeyModifiers::CONTROL => {
                    self.text_area
                        .set_yank_text(self.selected_range_to_string());
                    self.text_area.paste();
                    self.state.dirty = true;
                    return Ok(None);
                }
                KeyCode::Enter => self.exit_edit_mode(true)?,
                KeyCode::Esc => self.exit_edit_mode(false)?,
                _ => {
                    // NOOP
                }
            }
        }
        // TODO(zaphar): Some specialized editing keybinds
        // * Select All
        // * Copy
        // * Paste
        if self.text_area.input(key) {
            self.state.dirty = true;
        }
        Ok(None)
    }

    fn handle_command(&mut self, cmd_text: String) -> Result<Option<ExitCode>> {
        if cmd_text.is_empty() {
            return Ok(None);
        }
        match cmd::parse(&cmd_text) {
            Ok(Some(Cmd::Edit(path))) => {
                self.load_into(path)?;
                Ok(None)
            }
            Ok(Some(Cmd::Help(maybe_topic))) => {
                self.enter_dialog_mode(help::to_widget(maybe_topic.unwrap_or("")));
                Ok(None)
            }
            Ok(Some(Cmd::Write(maybe_path))) => {
                if let Some(path) = maybe_path {
                    self.save_to(path)?;
                } else {
                    self.save_file()?;
                }
                Ok(None)
            }
            Ok(Some(Cmd::ExportCsv(path))) => {
                self.book
                    .save_sheet_to_csv(self.book.location.sheet, path)?;
                Ok(None)
            }
            Ok(Some(Cmd::InsertColumns(count))) => {
                self.book.insert_columns(self.book.location.col, count)?;
                self.book.evaluate();
                Ok(None)
            }
            Ok(Some(Cmd::InsertRows(count))) => {
                self.book.insert_rows(self.book.location.row, count)?;
                self.book.evaluate();
                Ok(None)
            }
            Ok(Some(Cmd::RenameSheet(idx, name))) => {
                match idx {
                    Some(idx) => {
                        self.book.set_sheet_name(idx as u32, name)?;
                    }
                    _ => {
                        self.book.set_sheet_name(self.book.location.sheet, name)?;
                    }
                }
                Ok(None)
            }
            Ok(Some(Cmd::NewSheet(name))) => {
                self.book.new_sheet(name)?;
                Ok(None)
            }
            Ok(Some(Cmd::SelectSheet(name))) => {
                self.book.select_sheet_by_name(name);
                Ok(None)
            }
            Ok(Some(Cmd::Quit)) => self.quit_app(),
            Ok(Some(Cmd::ColorRows(count, color))) => {
                let row_count = count.unwrap_or(1);
                let row = self.book.location.row;
                for r in row..(row + row_count) {
                    self.book.set_row_style(
                        &[("fill.bg_color", &color)],
                        self.book.location.sheet,
                        r,
                    )?;
                }
                Ok(None)
            }
            Ok(Some(Cmd::ColorColumns(count, color))) => {
                let col_count = count.unwrap_or(1);
                let col = self.book.location.col;
                for c in col..(col + col_count) {
                    self.book.set_col_style(
                        &[("fill.bg_color", &color)],
                        self.book.location.sheet,
                        c,
                    )?;
                }
                Ok(None)
            }
            Ok(Some(Cmd::ColorCell(color))) => {
                let sheet = self.book.location.sheet;
                let area = if let Some((start, end)) = self.state.range_select.get_range() {
                    Area {
                        sheet,
                        row: start.row as i32,
                        column: start.col as i32,
                        width: (end.col - start.col + 1) as i32,
                        height: (end.row - start.row + 1) as i32,
                    }
                } else {
                    let address = self.book.location.clone();
                    Area {
                        sheet,
                        row: address.row as i32,
                        column: address.col as i32,
                        width: 1,
                        height: 1,
                    }
                };
                self.book
                    .set_cell_style(&[("fill.bg_color", &color)], &area)?;
                Ok(None)
            }
            Ok(Some(Cmd::SystemPaste)) => {
                let rows = self.get_rows_from_system_cipboard()?;
                self.state.clipboard = Some(ClipboardContents::Range(rows));
                self.paste_range()?;
                Ok(None)
            }
            Ok(None) => {
                self.enter_dialog_mode(Markdown::from_str(&format!(
                    "Unrecognized commmand {}",
                    cmd_text
                )));
                Ok(None)
            }
            Err(msg) => {
                self.enter_dialog_mode(Markdown::from_str(msg));
                Ok(None)
            }
        }
    }

    fn handle_numeric_prefix(&mut self, digit: char) {
        self.state.numeric_prefix.push(digit);
    }

    fn handle_range_select_input(&mut self, key: event::KeyEvent) -> Result<Option<ExitCode>> {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Esc => {
                    if self.state.numeric_prefix.len() > 0 {
                        self.state.reset_n_prefix();
                    } else {
                        self.state.range_select.start = None;
                        self.state.range_select.end = None;
                        self.exit_range_select_mode()?;
                    }
                }
                KeyCode::Char('h') if key.modifiers == KeyModifiers::ALT => {
                    self.enter_dialog_mode(self.render_help_text());
                    return Ok(None);
                }
                KeyCode::Char(d) if d.is_ascii_digit() => {
                    self.handle_numeric_prefix(d);
                }
                KeyCode::Char('D') => {
                    if let Some((start, end)) = self.state.range_select.get_range() {
                        self.book.clear_cell_range_all(start, end)?;
                    }
                }
                KeyCode::Char('d') => {
                    if let Some((start, end)) = self.state.range_select.get_range() {
                        self.book.clear_cell_range(start, end)?;
                    }
                }
                KeyCode::Char('h') => {
                    self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                        ws.move_left()?;
                        Ok(())
                    })?;
                    self.maybe_update_range_end();
                }
                KeyCode::Char('j') => {
                    self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                        ws.move_down()?;
                        Ok(())
                    })?;
                    self.maybe_update_range_end();
                }
                KeyCode::Char('k') => {
                    self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                        ws.move_up()?;
                        Ok(())
                    })?;
                    self.maybe_update_range_end();
                }
                KeyCode::Char('l') => {
                    self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                        ws.move_right()?;
                        Ok(())
                    })?;
                    self.maybe_update_range_end();
                }
                KeyCode::Char(' ') | KeyCode::Enter => {
                    if self.update_range_selection()? {
                        self.exit_range_select_mode()?;
                    }
                }
                KeyCode::Char('n') if key.modifiers == KeyModifiers::CONTROL => {
                    self.state.range_select.reset_range_selection();
                    self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                        ws.book.select_next_sheet();
                        Ok(())
                    })?;
                }
                KeyCode::Char('p') if key.modifiers == KeyModifiers::CONTROL => {
                    self.state.range_select.reset_range_selection();
                    self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                        ws.book.select_prev_sheet();
                        Ok(())
                    })?;
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.copy_range(true)?;
                    self.exit_range_select_mode()?;
                }
                KeyCode::Char('y') => {
                    self.copy_range(true)?;
                    self.exit_range_select_mode()?;
                }
                KeyCode::Char('C') if key.modifiers == KeyModifiers::CONTROL => {
                    self.copy_range(false)?;
                    self.exit_range_select_mode()?;
                }
                KeyCode::Char('Y') => {
                    self.copy_range(false)?;
                    self.exit_range_select_mode()?;
                }
                KeyCode::Char('x') => {
                    if let (Some(from), Some(to)) = (
                        self.state.range_select.start.as_ref(),
                        self.state.range_select.end.as_ref(),
                    ) {
                        self.book.extend_to(from, to)?;
                    }
                    self.exit_range_select_mode()?;
                }
                KeyCode::Char(':') => {
                    self.enter_command_mode();
                }
                _ => {
                    // moop
                }
            }
        }
        Ok(None)
    }

    fn copy_range(&mut self, formatted: bool) -> Result<(), anyhow::Error> {
        use arboard::Clipboard;
        self.update_range_selection()?;
        match &self.state.range_select.get_range() {
            Some((start, end)) => {
                let mut rows = Vec::new();
                for row in (AddressRange { start, end }).as_rows() {
                    let mut cols = Vec::new();
                    for cell in row {
                        cols.push(if formatted {
                            self.book.get_cell_addr_rendered(&cell)?
                        } else {
                            self.book.get_cell_addr_contents(&cell)?
                        });
                    }
                    rows.push(cols);
                }
                // TODO(zaphar): Rethink this a bit perhaps?
                let mut cb = Clipboard::new()?;
                let (html, csv) = self
                    .book
                    .range_to_clipboard_content(AddressRange { start, end })?;
                cb.set_html(html, Some(csv))?;
                self.state.clipboard = Some(ClipboardContents::Range(rows));
            }
            None => {
                self.state.clipboard = Some(ClipboardContents::Cell(if formatted {
                    self.book.get_current_cell_rendered()?
                } else {
                    self.book.get_current_cell_contents()?
                }));
            }
        }
        Ok(())
    }

    fn get_rows_from_system_cipboard(&mut self) -> Result<Vec<Vec<String>>, anyhow::Error> {
        use arboard::Clipboard;
        let mut cb = Clipboard::new()?;
        let txt = match cb.get_text() {
            Ok(txt) => txt,
            Err(e) => return Err(anyhow!(e)),
        };
        let reader = csv::Reader::from_reader(txt.as_bytes());
        let mut rows = Vec::new();
        for rec in reader.into_byte_records() {
            let record = rec?;
            let mut row = Vec::with_capacity(record.len());
            for i in 0..record.len() {
                row.push(String::from_utf8_lossy(record.get(i).expect("Unexpected failure to get cell row")).to_string());
            };
            rows.push(row);
        }
        Ok(rows)
    }

    fn update_range_selection(&mut self) -> Result<bool, anyhow::Error> {
        Ok(if self.state.range_select.start.is_none() {
            self.state.range_select.start = Some(self.book.location.clone());
            self.state.range_select.end = Some(self.book.location.clone());
            false
        } else {
            self.state.range_select.end = Some(self.book.location.clone());
            true
        })
    }

    fn maybe_update_range_end(&mut self) {
        if self.state.range_select.start.is_some() {
            self.state.range_select.end = Some(self.book.location.clone());
        }
    }

    fn handle_navigation_input(&mut self, key: event::KeyEvent) -> Result<Option<ExitCode>> {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Esc => {
                    self.state.reset_n_prefix();
                    self.state.char_queue.clear();
                }
                KeyCode::Char('B') => {
                    let address = self.book.location.clone();
                    let style = self.book.get_cell_style(&address).map(|s| s.font.b);
                    self.toggle_bool_style(style, "font.b", &address)?;
                }
                KeyCode::Char('I') => {
                    let address = self.book.location.clone();
                    let style = self.book.get_cell_style(&address).map(|s| s.font.i);
                    self.toggle_bool_style(style, "font.i", &address)?;
                }
                KeyCode::Char(d) if d.is_ascii_digit() => {
                    self.handle_numeric_prefix(d);
                }
                KeyCode::Char('e') | KeyCode::Char('i') => {
                    self.enter_edit_mode();
                }
                KeyCode::Char(':') => {
                    self.enter_command_mode();
                }
                KeyCode::Char('s') if key.modifiers == KeyModifiers::CONTROL => {
                    self.save_file()?;
                }
                KeyCode::Char('s') if key.modifiers != KeyModifiers::CONTROL => {
                    self.book.clear_current_cell()?;
                    self.text_area = reset_text_area(String::new());
                    self.enter_edit_mode();
                }
                KeyCode::Char('r') if key.modifiers == KeyModifiers::CONTROL => {
                    self.enter_range_select_mode(false);
                }
                KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                    self.state.clipboard = Some(ClipboardContents::Cell(
                        self.book.get_current_cell_contents()?,
                    ));
                }
                KeyCode::Char('y') => {
                    self.state.clipboard = Some(ClipboardContents::Cell(
                        self.book.get_current_cell_contents()?,
                    ));
                }
                KeyCode::Char('Y') => {
                    self.state.clipboard = Some(ClipboardContents::Cell(
                        self.book.get_current_cell_rendered()?,
                    ));
                }
                KeyCode::Char('C') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.state.clipboard = Some(ClipboardContents::Cell(
                        self.book.get_current_cell_rendered()?,
                    ));
                }
                KeyCode::Char('v') if key.modifiers != KeyModifiers::CONTROL => {
                    self.enter_range_select_mode(true)
                }
                KeyCode::Char('p') if key.modifiers != KeyModifiers::CONTROL => {
                    self.paste_range()?;
                }
                KeyCode::Char('v') if key.modifiers == KeyModifiers::CONTROL => {
                    self.paste_range()?;
                }
                KeyCode::Char('h') if key.modifiers == KeyModifiers::ALT => {
                    self.enter_dialog_mode(self.render_help_text());
                }
                KeyCode::Char('n') if key.modifiers == KeyModifiers::CONTROL => {
                    self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                        ws.book.select_next_sheet();
                        Ok(())
                    })?;
                }
                KeyCode::Char('d') => {
                    self.book.clear_current_cell()?;
                }
                KeyCode::Char('D') => {
                    self.book.clear_current_cell_all()?;
                }
                KeyCode::Char('p') if key.modifiers == KeyModifiers::CONTROL => {
                    self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                        ws.book.select_prev_sheet();
                        Ok(())
                    })?;
                }
                KeyCode::Char('l') if key.modifiers == KeyModifiers::CONTROL => {
                    self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                        let Address {
                            sheet: _,
                            row: _,
                            col,
                        } = &ws.book.location;
                        ws.book
                            .set_col_size(*col, ws.book.get_col_size(*col)? + 1)?;
                        Ok(())
                    })?;
                }
                KeyCode::Char('h') if key.modifiers == KeyModifiers::CONTROL => {
                    self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                        let Address {
                            sheet: _,
                            row: _,
                            col,
                        } = &ws.book.location;
                        let curr_size = ws.book.get_col_size(*col)?;
                        if curr_size > 1 {
                            ws.book.set_col_size(*col, curr_size - 1)?;
                        }
                        Ok(())
                    })?;
                }
                KeyCode::Char('q') => {
                    return self.quit_app();
                }
                KeyCode::Char('j') | KeyCode::Down if key.modifiers != KeyModifiers::CONTROL => {
                    self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                        ws.move_down()?;
                        ws.handle_movement_change();
                        Ok(())
                    })?;
                }
                KeyCode::Enter if key.modifiers != KeyModifiers::SHIFT => {
                    self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                        ws.move_down()?;
                        ws.handle_movement_change();
                        Ok(())
                    })?;
                }
                KeyCode::Enter if key.modifiers == KeyModifiers::SHIFT => {
                    self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                        ws.move_up()?;
                        ws.handle_movement_change();
                        Ok(())
                    })?;
                }
                KeyCode::Char('k') | KeyCode::Up if key.modifiers != KeyModifiers::CONTROL => {
                    self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                        ws.move_up()?;
                        ws.handle_movement_change();
                        Ok(())
                    })?;
                }
                KeyCode::Char('h') | KeyCode::Left if key.modifiers != KeyModifiers::CONTROL => {
                    self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                        ws.move_left()?;
                        ws.handle_movement_change();
                        Ok(())
                    })?;
                }
                KeyCode::Char('l') | KeyCode::Right if key.modifiers != KeyModifiers::CONTROL => {
                    self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                        ws.move_right()?;
                        ws.handle_movement_change();
                        Ok(())
                    })?;
                }
                KeyCode::Tab if key.modifiers != KeyModifiers::SHIFT => {
                    self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                        ws.move_right()?;
                        ws.handle_movement_change();
                        Ok(())
                    })?;
                }
                KeyCode::Tab if key.modifiers == KeyModifiers::SHIFT => {
                    self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                        ws.move_left()?;
                        ws.handle_movement_change();
                        Ok(())
                    })?;
                }
                KeyCode::Char('g') => {
                    // TODO(zaphar): This really needs a better state machine.
                    if self
                        .state
                        .char_queue
                        .first()
                        .map(|c| *c == 'g')
                        .unwrap_or(false)
                    {
                        self.state.char_queue.pop();
                        self.move_to_top()?;
                    } else {
                        self.state.char_queue.push('g');
                    }
                }
                _ => {
                    // noop
                    self.state.char_queue.clear();
                }
            }
        }
        return Ok(None);
    }

    fn toggle_bool_style(
        &mut self,
        current_val: Option<bool>,
        path: &str,
        address: &Address,
    ) -> Result<(), anyhow::Error> {
        let value = if let Some(b_val) = current_val {
            if b_val {
                "false"
            } else {
                "true"
            }
        } else {
            "true"
        };
        self.book.set_cell_style(
            &[(path, value)],
            &Area {
                sheet: address.sheet,
                row: address.row as i32,
                column: address.col as i32,
                width: 1,
                height: 1,
            },
        )?;
        Ok(())
    }

    fn paste_range(&mut self) -> Result<(), anyhow::Error> {
        match &self.state.clipboard {
            Some(ClipboardContents::Cell(contents)) => {
                self.book.edit_current_cell(contents)?;
                self.book.evaluate();
            }
            Some(ClipboardContents::Range(ref rows)) => {
                let Address { sheet, row, col } = self.book.location.clone();
                let row_len = rows.len();
                for ri in 0..row_len {
                    let columns = &rows[ri];
                    let col_len = columns.len();
                    for ci in 0..col_len {
                        self.book.update_cell(
                            &Address {
                                sheet,
                                row: ri + row,
                                col: ci + col,
                            },
                            columns[ci].clone(),
                        )?;
                    }
                }
                self.book.evaluate();
            }
            None => {
                // NOOP
            }
        }
        self.state.clipboard = None;
        Ok(())
    }

    fn run_with_prefix(
        &mut self,
        action: impl Fn(&mut Workspace<'_>) -> std::result::Result<(), anyhow::Error>,
    ) -> Result<(), anyhow::Error> {
        for _ in 1..=self.state.get_n_prefix() {
            action(self)?;
        }
        self.state.reset_n_prefix();
        Ok(())
    }

    fn enter_quit_mode(&mut self) -> bool {
        if self.book.dirty {
            self.state.modality_stack.push(Modality::Quit);
            return true;
        }
        return false;
    }

    fn enter_command_mode(&mut self) {
        self.state.modality_stack.push(Modality::Command);
        self.state.command_state.truncate();
        *self.state.command_state.status_mut() = Status::Pending;
        self.state.command_state.focus();
    }

    fn enter_dialog_mode(&mut self, msg: Markdown) {
        self.state.popup = Some(msg);
        self.state.modality_stack.push(Modality::Dialog);
    }

    fn enter_range_select_mode(&mut self, init_start: bool) {
        self.state.range_select.original_location = Some(self.book.location.clone());
        if init_start {
            self.state.range_select.start = Some(self.book.location.clone());
        } else {
            self.state.range_select.start = None;
        }
        self.state.range_select.end = None;
        self.state.modality_stack.push(Modality::RangeSelect);
    }

    fn enter_edit_mode(&mut self) {
        self.state.modality_stack.push(Modality::CellEdit);
        self.text_area
            .set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
        self.text_area
            .set_cursor_style(Style::default().add_modifier(Modifier::SLOW_BLINK));
        self.text_area.move_cursor(CursorMove::Bottom);
        self.text_area.move_cursor(CursorMove::End);
    }

    fn exit_quit_mode(&mut self) -> Result<Option<ExitCode>> {
        self.state.pop_modality();
        Ok(None)
    }

    fn exit_command_mode(&mut self) -> Result<Option<ExitCode>> {
        let cmd = self.state.command_state.value().to_owned();
        self.state.command_state.blur();
        *self.state.command_state.status_mut() = Status::Done;
        self.state.pop_modality();
        self.handle_command(cmd)
    }

    fn exit_dialog_mode(&mut self) -> Result<()> {
        self.state.pop_modality();
        Ok(())
    }

    fn exit_range_select_mode(&mut self) -> Result<()> {
        self.book.location = self
            .state
            .range_select
            .original_location
            .clone()
            .expect("Missing original location after range copy");
        self.state.range_select.original_location = None;
        self.state.pop_modality();
        if self.state.modality() == &Modality::CellEdit {
            self.text_area
                .set_yank_text(self.selected_range_to_string());
            self.text_area.paste();
            self.state.dirty = true;
        }
        Ok(())
    }

    fn exit_edit_mode(&mut self, keep: bool) -> Result<()> {
        self.text_area.set_cursor_line_style(Style::default());
        self.text_area.set_cursor_style(Style::default());
        let contents = self.text_area.lines().join("\n");
        if self.state.dirty && keep {
            self.book.edit_current_cell(contents)?;
            self.book.evaluate();
        }
        self.text_area = reset_text_area(self.book.get_current_cell_contents()?);
        self.state.dirty = false;
        self.state.pop_modality();
        Ok(())
    }

    fn handle_movement_change(&mut self) {
        let contents = self
            .book
            .get_current_cell_contents()
            .expect("Unexpected failure getting current cell contents");
        self.text_area = reset_text_area(contents);
    }

    fn save_file(&mut self) -> Result<()> {
        self.book
            .save_to_xlsx(&self.name.to_string_lossy().to_string())?;
        Ok(())
    }

    fn save_to<S: Into<String>>(&mut self, path: S) -> Result<()> {
        self.book.save_to_xlsx(path.into().as_str())?;
        Ok(())
    }

    fn quit_app(&mut self) -> std::result::Result<Option<ExitCode>, anyhow::Error> {
        if self.enter_quit_mode() {
            return Ok(None);
        }
        return Ok(Some(ExitCode::SUCCESS));
    }
}

fn load_book(path: &PathBuf, locale: &str, tz: &str) -> Result<Book, anyhow::Error> {
    let book = if path.exists() {
        Book::new_from_xlsx_with_locale(&path.to_string_lossy().to_string(), locale, tz)?
    } else {
        Book::default()
    };
    Ok(book)
}

fn reset_text_area<'a>(content: String) -> TextArea<'a> {
    let mut text_area = TextArea::from(content.lines());
    text_area.set_cursor_line_style(Style::default());
    text_area.set_cursor_style(Style::default());
    text_area.set_block(Block::bordered());
    text_area
}
