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
}

#[derive(Debug)]
pub struct AppState<'ws> {
    pub modality_stack: Vec<Modality>,
    pub viewport_state: ViewportState,
    pub command_state: TextState<'ws>,
    dirty: bool,
    popup: Vec<String>,
}

impl<'ws> Default for AppState<'ws> {
    fn default() -> Self {
        AppState {
            modality_stack: vec![Modality::default()],
            viewport_state: Default::default(),
            command_state: Default::default(),
            dirty: Default::default(),
            popup: Default::default(),
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
                "* CTRl-r: Add a row".to_string(),
                "* CTRl-c: Add a column".to_string(),
                "* CTRl-l: Grow column width by 1".to_string(),
                "* CTRl-h: Shrink column width by 1".to_string(),
                "* CTRl-n: Next sheet. Starts over at beginning if at end.".to_string(),
                "* CTRl-p: Previous sheet. Starts over at end if at beginning.".to_string(),
                "* q exit".to_string(),
                "* Ctrl-S Save sheet".to_string(),
            ],
            Modality::CellEdit => vec![
                "Edit Mode:".to_string(),
                "* ESC, ENTER/RETURN: Exit edit mode".to_string(),
                "Otherwise edit as normal".to_string(),
            ],
            Modality::Command => vec![
                "Command Mode:".to_string(),
                "* ESC: Exit command mode".to_string(),
                "* ENTER/RETURN: run command and exit command mode".to_string(),
            ],
            _ => vec!["General help".to_string()],
        }
    }

    fn handle_command_input(&mut self, key: event::KeyEvent) -> Result<Option<ExitCode>> {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => self.exit_command_mode()?,
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
                KeyCode::Char('?') => {
                    self.enter_dialog_mode(self.render_help_text());
                }
                KeyCode::Esc | KeyCode::Enter => self.exit_edit_mode()?,
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

    fn handle_command(&mut self, cmd_text: String) -> Result<bool> {
        if cmd_text.is_empty() {
            return Ok(true);
        }
        match cmd::parse(&cmd_text) {
            Ok(Some(Cmd::Edit(path))) => {
                self.load_into(path)?;
                Ok(true)
            }
            Ok(Some(Cmd::Help(_maybe_topic))) => {
                self.enter_dialog_mode(vec!["TODO help topic".to_owned()]);
                Ok(true)
            }
            Ok(Some(Cmd::Write(maybe_path))) => {
                if let Some(path) = maybe_path {
                    self.save_to(path)?;
                } else {
                    self.save_file()?;
                }
                Ok(true)
            }
            Ok(Some(Cmd::InsertColumns(count))) => {
                self.book.insert_columns(self.book.location.col, count)?;
                self.book.evaluate();
                Ok(true)
            }
            Ok(Some(Cmd::InsertRow(count))) => {
                self.book.insert_rows(self.book.location.row, count)?;
                self.book.evaluate();
                Ok(true)
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
                Ok(true)
            }
            Ok(Some(Cmd::NewSheet(name))) => {
                self.book.new_sheet(name)?;
                Ok(true)
            }
            Ok(Some(Cmd::SelectSheet(name))) => {
                self.book.select_sheet_by_name(name);
                Ok(true)
            }
            Ok(Some(Cmd::Quit)) => {
                // TODO(zaphar): We probably need to do better than this
                std::process::exit(0);
            }
            Ok(None) => {
                self.enter_dialog_mode(vec![format!("Unrecognized commmand {}", cmd_text)]);
                Ok(false)
            }
            Err(msg) => {
                self.enter_dialog_mode(vec![msg.to_owned()]);
                Ok(false)
            }
        }
    }

    fn handle_navigation_input(&mut self, key: event::KeyEvent) -> Result<Option<ExitCode>> {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Char('e') | KeyCode::Char('i') => {
                    self.enter_edit_mode();
                }
                KeyCode::Char(':') => {
                    self.enter_command_mode();
                }
                KeyCode::Char('s') if key.modifiers == KeyModifiers::CONTROL => {
                    self.save_file()?;
                }
                KeyCode::Char('?') => {
                    self.enter_dialog_mode(self.render_help_text());
                }
                KeyCode::Char('n') if key.modifiers == KeyModifiers::CONTROL => {
                    self.book.select_next_sheet();
                }
                KeyCode::Char('p') if key.modifiers == KeyModifiers::CONTROL => {
                    self.book.select_prev_sheet();
                }
                KeyCode::Char('s')
                    if key.modifiers == KeyModifiers::HYPER
                        || key.modifiers == KeyModifiers::SUPER =>
                {
                    self.save_file()?;
                }
                KeyCode::Char('l') if key.modifiers == KeyModifiers::CONTROL => {
                    let Address { row: _, col } = &self.book.location;
                    self.book
                        .set_col_size(*col, self.book.get_col_size(*col)? + 1)?;
                }
                KeyCode::Char('h') if key.modifiers == KeyModifiers::CONTROL => {
                    let Address { row: _, col } = &self.book.location;
                    let curr_size = self.book.get_col_size(*col)?;
                    if curr_size > 1 {
                        self.book.set_col_size(*col, curr_size - 1)?;
                    }
                }
                KeyCode::Char('r') if key.modifiers == KeyModifiers::CONTROL => {
                    let (row_count, _) = self.book.get_size()?;
                    self.book.update_entry(
                        &Address {
                            row: row_count + 1,
                            col: 1,
                        },
                        "",
                    )?;
                    let (row, _) = self.book.get_size()?;
                    let mut loc = self.book.location.clone();
                    if loc.row < row as usize {
                        loc.row = row as usize;
                        self.book.move_to(&loc)?;
                    }
                    self.handle_movement_change();
                }
                KeyCode::Char('t') if key.modifiers == KeyModifiers::CONTROL => {
                    let (_, col_count) = self.book.get_size()?;
                    self.book.update_entry(
                        &Address {
                            row: 1,
                            col: col_count + 1,
                        },
                        "",
                    )?;
                }
                KeyCode::Char('q') => {
                    return Ok(Some(ExitCode::SUCCESS));
                }
                KeyCode::Char('j') | KeyCode::Down
                    if key.modifiers != KeyModifiers::CONTROL =>
                {
                    self.move_down()?;
                    self.handle_movement_change();
                }
                KeyCode::Enter
                    if key.modifiers != KeyModifiers::SHIFT =>
                {
                    self.move_down()?;
                    self.handle_movement_change();
                }
                KeyCode::Enter
                    if key.modifiers == KeyModifiers::SHIFT =>
                {
                    self.move_up()?;
                    self.handle_movement_change();
                }
                KeyCode::Char('k') | KeyCode::Up if key.modifiers != KeyModifiers::CONTROL => {
                    self.move_up()?;
                    self.handle_movement_change();
                }
                KeyCode::Char('h') | KeyCode::Left if key.modifiers != KeyModifiers::CONTROL => {
                    self.move_left()?;
                    self.handle_movement_change();
                }
                KeyCode::Char('l') | KeyCode::Right
                    if key.modifiers != KeyModifiers::CONTROL =>
                {
                    self.move_right()?;
                    self.handle_movement_change();
                }
                KeyCode::Tab
                    if key.modifiers != KeyModifiers::SHIFT =>
                {
                    self.move_right()?;
                    self.handle_movement_change();
                }
                KeyCode::Tab
                    if key.modifiers == KeyModifiers::SHIFT =>
                {
                    self.move_left()?;
                    self.handle_movement_change();
                }
                _ => {
                    // noop
                }
            }
        }
        return Ok(None);
    }

    fn enter_navigation_mode(&mut self) {
        self.state.modality_stack.push(Modality::Navigate);
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

    fn enter_edit_mode(&mut self) {
        self.state.modality_stack.push(Modality::CellEdit);
        self.text_area
            .set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
        self.text_area
            .set_cursor_style(Style::default().add_modifier(Modifier::SLOW_BLINK));
        self.text_area.move_cursor(CursorMove::Bottom);
        self.text_area.move_cursor(CursorMove::End);
    }

    fn exit_command_mode(&mut self) -> Result<()> {
        let cmd = self.state.command_state.value().to_owned();
        self.state.command_state.blur();
        *self.state.command_state.status_mut() = Status::Done;
        self.state.pop_modality();
        self.handle_command(cmd)?;
        Ok(())
    }

    fn exit_dialog_mode(&mut self) -> Result<()> {
        self.state.pop_modality();
        Ok(())
    }

    fn exit_edit_mode(&mut self) -> Result<()> {
        self.text_area.set_cursor_line_style(Style::default());
        self.text_area.set_cursor_style(Style::default());
        let contents = self.text_area.lines().join("\n");
        if self.state.dirty {
            self.book.edit_current_cell(contents)?;
            self.book.evaluate();
            self.state.dirty = false;
        }
        self.enter_navigation_mode();
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
