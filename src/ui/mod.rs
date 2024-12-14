//! Ui rendering logic
use std::{path::PathBuf, process::ExitCode};

use crate::book::Book;

use anyhow::{anyhow, Result};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ironcalc::base::Model;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout},
    style::{Modifier, Style},
    widgets::Block,
};
use tui_prompts::{State, Status, TextPrompt, TextState};
use tui_textarea::{CursorMove, TextArea};

mod cmd;
pub mod render;
#[cfg(test)]
mod test;

use cmd::Cmd;
use render::viewport::ViewportState;

#[derive(Default, Debug, PartialEq, Clone)]
pub enum Modality {
    #[default]
    Navigate,
    CellEdit,
    Command,
    Dialog,
    RangeSelect,
}

#[derive(Debug, Default)]
pub struct RangeSelection {
    pub original_location: Option<Address>,
    pub original_sheet: Option<u32>,
    pub sheet: Option<u32>,
    pub start: Option<Address>,
    pub end: Option<Address>,
}

impl RangeSelection {
    pub fn get_range(&self) -> Option<(Address, Address)> {
        if let (Some(start), Some(end)) = (&self.start, &self.end) {
            return Some((
                Address {
                    row: std::cmp::min(start.row, end.row),
                    col: std::cmp::min(start.col, end.col),
                },
                Address {
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
        self.sheet = None;
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
    pub range_select: RangeSelection,
    dirty: bool,
    popup: Vec<String>,
    clipboard: Option<ClipboardContents>,
}

impl<'ws> Default for AppState<'ws> {
    fn default() -> Self {
        AppState {
            modality_stack: vec![Modality::default()],
            viewport_state: Default::default(),
            command_state: Default::default(),
            numeric_prefix: Default::default(),
            range_select: Default::default(),
            dirty: Default::default(),
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

// TODO(jwall): This should probably move to a different module.
/// The Address in a Table.
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone)]
pub struct Address {
    pub row: usize,
    pub col: usize,
}

impl Address {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
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
            Book::new(Model::new_empty("", locale, tz).map_err(|e| anyhow!("{}", e))?),
            PathBuf::default(),
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
            if let Some(range_sheet) = state.range_select.sheet {
                if range_sheet != self.book.current_sheet {
                    return format!(
                        "{}!{}",
                        self.book
                            .get_sheet_name_by_idx(range_sheet as usize)
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
        if loc.row < render::viewport::LAST_ROW {
            loc.row += 1;
            self.book.move_to(&loc)?;
        }
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
        if loc.col < render::viewport::LAST_COLUMN {
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
            };
            return Ok(result);
        }
        Ok(None)
    }

    fn render_help_text(&self) -> Vec<String> {
        match self.state.modality() {
            Modality::Navigate => vec![
                "Navigate Mode:".to_string(),
                "* e,i: Enter edit mode for current cell".to_string(),
                "* ENTER/RETURN: Go down one cell".to_string(),
                "* TAB: Go over one cell".to_string(),
                "* h,j,k,l: vim style navigation".to_string(),
                "* d: clear cell contents leaving style untouched".to_string(),
                "* D: clear cell contents including style".to_string(),
                "* CTRl-r: Add a row".to_string(),
                "* CTRl-c: Add a column".to_string(),
                "* CTRl-l: Grow column width by 1".to_string(),
                "* CTRl-h: Shrink column width by 1".to_string(),
                "* CTRl-n: Next sheet. Starts over at beginning if at end.".to_string(),
                "* CTRl-p: Previous sheet. Starts over at end if at beginning.".to_string(),
                "* ALT-h: Previous sheet. Starts over at end if at beginning.".to_string(),
                "* q exit".to_string(),
                "* Ctrl-S Save sheet".to_string(),
            ],
            Modality::CellEdit => vec![
                "Edit Mode:".to_string(),
                "* ENTER/RETURN: Exit edit mode and save changes".to_string(),
                "* Ctrl-r: Enter Range Selection mode".to_string(),
                "* ESC: Exit edit mode and discard changes".to_string(),
                "Otherwise edit as normal".to_string(),
            ],
            Modality::Command => vec![
                "Command Mode:".to_string(),
                "* ESC: Exit command mode".to_string(),
                "* CTRL-?: Exit command mode".to_string(),
                "* ENTER/RETURN: run command and exit command mode".to_string(),
            ],
            Modality::RangeSelect => vec![
                "Range Selection Mode:".to_string(),
                "* ESC: Exit command mode".to_string(),
                "* h,j,k,l: vim style navigation".to_string(),
                "* d: delete the contents of the range leaving style untouched".to_string(),
                "* D: clear cell contents including style".to_string(),
                "* Spacebar: Select start and end of range".to_string(),
                "* CTRl-n: Next sheet. Starts over at beginning if at end.".to_string(),
                "* CTRl-p: Previous sheet. Starts over at end if at beginning.".to_string(),
            ],
            _ => vec!["General help".to_string()],
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

    fn handle_dialog_input(&mut self, key: event::KeyEvent) -> Result<Option<ExitCode>> {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => self.exit_dialog_mode()?,
                KeyCode::Char('h') if key.modifiers == KeyModifiers::ALT => {
                    self.exit_dialog_mode()?
                }
                _ => {
                    // NOOP
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
                    self.enter_range_select_mode();
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
            Ok(Some(Cmd::Help(_maybe_topic))) => {
                self.enter_dialog_mode(vec!["TODO help topic".to_owned()]);
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
            Ok(Some(Cmd::InsertColumns(count))) => {
                self.book.insert_columns(self.book.location.col, count)?;
                self.book.evaluate();
                Ok(None)
            }
            Ok(Some(Cmd::InsertRow(count))) => {
                self.book.insert_rows(self.book.location.row, count)?;
                self.book.evaluate();
                Ok(None)
            }
            Ok(Some(Cmd::RenameSheet(idx, name))) => {
                match idx {
                    Some(idx) => {
                        self.book.set_sheet_name(idx, name)?;
                    }
                    _ => {
                        self.book
                            .set_sheet_name(self.book.current_sheet as usize, name)?;
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
            Ok(Some(Cmd::Quit)) => {
                Ok(Some(ExitCode::SUCCESS))
            }
            Ok(None) => {
                self.enter_dialog_mode(vec![format!("Unrecognized commmand {}", cmd_text)]);
                Ok(None)
            }
            Err(msg) => {
                self.enter_dialog_mode(vec![msg.to_owned()]);
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
                        self.book.clear_cell_range_all(
                            self.state
                                .range_select
                                .sheet
                                .unwrap_or_else(|| self.book.current_sheet),
                            start,
                            end,
                        )?;
                    }
                }
                KeyCode::Char('d') => {
                    if let Some((start, end)) = self.state.range_select.get_range() {
                        self.book.clear_cell_range(
                            self.state
                                .range_select
                                .sheet
                                .unwrap_or_else(|| self.book.current_sheet),
                            start,
                            end,
                        )?;
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
                    self.update_range_selection()?;
                }
                KeyCode::Char('n') if key.modifiers == KeyModifiers::CONTROL => {
                    self.state.range_select.reset_range_selection();
                    self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                        ws.book.select_next_sheet();
                        Ok(())
                    })?;
                    self.state.range_select.sheet = Some(self.book.current_sheet);
                }
                KeyCode::Char('p') if key.modifiers == KeyModifiers::CONTROL => {
                    self.state.range_select.reset_range_selection();
                    self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                        ws.book.select_prev_sheet();
                        Ok(())
                    })?;
                    self.state.range_select.sheet = Some(self.book.current_sheet);
                }
                KeyCode::Char('C')
                    if key
                        .modifiers
                        .contains(KeyModifiers::CONTROL | KeyModifiers::SHIFT) =>
                {
                    // TODO(zaphar): Share the algorithm below between both copies
                    self.copy_range(true)?;
                }
                KeyCode::Char('Y') => self.copy_range(true)?,
                KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                    self.copy_range(false)?;
                }
                KeyCode::Char('y') => self.copy_range(false)?,
                _ => {
                    // moop
                }
            }
        }
        Ok(None)
    }

    fn copy_range(&mut self, formatted: bool) -> Result<(), anyhow::Error> {
        self.update_range_selection()?;
        match &self.state.range_select.get_range() {
            Some((
                Address {
                    row: row_start,
                    col: col_start,
                },
                Address {
                    row: row_end,
                    col: col_end,
                },
            )) => {
                let mut rows = Vec::new();
                for ri in (*row_start)..=(*row_end) {
                    let mut cols = Vec::new();
                    for ci in (*col_start)..=(*col_end) {
                        cols.push(if formatted {
                            self.book
                                .get_cell_addr_rendered(&Address { row: ri, col: ci })?
                        } else {
                            self.book
                                .get_cell_addr_contents(&Address { row: ri, col: ci })?
                        });
                    }
                    rows.push(cols);
                }
                self.state.clipboard = Some(ClipboardContents::Range(rows));
            }
            None => {
                self.state.clipboard = Some(ClipboardContents::Cell(if formatted {
                    self.book
                        .get_current_cell_rendered()?
                } else {
                    self.book
                        .get_current_cell_contents()?
                }));
            }
        }
        self.exit_range_select_mode()?;
        Ok(())
    }

    fn update_range_selection(&mut self) -> Result<(), anyhow::Error> {
        Ok(if self.state.range_select.start.is_none() {
            self.state.range_select.start = Some(self.book.location.clone());
            self.state.range_select.end = Some(self.book.location.clone());
        } else {
            self.state.range_select.end = Some(self.book.location.clone());
            self.exit_range_select_mode()?;
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
                KeyCode::Char('r') if key.modifiers == KeyModifiers::CONTROL => {
                    self.enter_range_select_mode();
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
                KeyCode::Char('C')
                    if key
                        .modifiers
                        .contains(KeyModifiers::CONTROL | KeyModifiers::SHIFT) =>
                {
                    self.state.clipboard = Some(ClipboardContents::Cell(
                        self.book.get_current_cell_rendered()?,
                    ));
                }
                KeyCode::Char('v') if key.modifiers != KeyModifiers::CONTROL => {
                    self.enter_range_select_mode()
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
                KeyCode::Char('s')
                    if key.modifiers == KeyModifiers::HYPER
                        || key.modifiers == KeyModifiers::SUPER =>
                {
                    self.save_file()?;
                }
                KeyCode::Char('l') if key.modifiers == KeyModifiers::CONTROL => {
                    self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                        let Address { row: _, col } = &ws.book.location;
                        ws.book
                            .set_col_size(*col, ws.book.get_col_size(*col)? + 1)?;
                        Ok(())
                    })?;
                }
                KeyCode::Char('h') if key.modifiers == KeyModifiers::CONTROL => {
                    self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                        let Address { row: _, col } = &ws.book.location;
                        let curr_size = ws.book.get_col_size(*col)?;
                        if curr_size > 1 {
                            ws.book.set_col_size(*col, curr_size - 1)?;
                        }
                        Ok(())
                    })?;
                }
                KeyCode::Char('q') => {
                    return Ok(Some(ExitCode::SUCCESS));
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
                _ => {
                    // noop
                }
            }
        }
        return Ok(None);
    }

    fn paste_range(&mut self) -> Result<(), anyhow::Error> {
        match &self.state.clipboard {
            Some(ClipboardContents::Cell(contents)) => {
                self.book.edit_current_cell(contents)?;
            }
            Some(ClipboardContents::Range(ref rows)) => {
                let Address { row, col } = self.book.location.clone();
                let row_len = rows.len();
                for ri in 0..row_len {
                    let columns = &rows[ri];
                    let col_len = columns.len();
                    for ci in 0..col_len {
                        self.book.update_cell(
                            &Address {
                                row: ri + row,
                                col: ci + col,
                            },
                            columns[ci].clone(),
                        )?;
                    }
                }
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

    fn enter_command_mode(&mut self) {
        self.state.modality_stack.push(Modality::Command);
        self.state.command_state.truncate();
        *self.state.command_state.status_mut() = Status::Pending;
        self.state.command_state.focus();
    }

    fn enter_dialog_mode(&mut self, msg: Vec<String>) {
        self.state.popup = msg;
        self.state.modality_stack.push(Modality::Dialog);
    }

    fn enter_range_select_mode(&mut self) {
        self.state.range_select.sheet = Some(self.book.current_sheet);
        self.state.range_select.original_sheet = Some(self.book.current_sheet);
        self.state.range_select.original_location = Some(self.book.location.clone());
        self.state.range_select.start = None;
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
        self.book.current_sheet = self
            .state
            .range_select
            .original_sheet
            .clone()
            .expect("Missing original sheet");
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

    fn save_file(&self) -> Result<()> {
        self.book
            .save_to_xlsx(&self.name.to_string_lossy().to_string())?;
        Ok(())
    }

    fn save_to<S: Into<String>>(&self, path: S) -> Result<()> {
        self.book.save_to_xlsx(path.into().as_str())?;
        Ok(())
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
