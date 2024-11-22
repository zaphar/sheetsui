//! Ui rendering logic

use std::{path::PathBuf, process::ExitCode};

use crate::book::Book;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    self,
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Cell, Paragraph, Row, Table, TableState, Widget},
    Frame,
};
use tui_prompts::{State, Status, TextPrompt, TextState};
use tui_textarea::{CursorMove, TextArea};

mod cmd;
#[cfg(test)]
mod test;

use cmd::Cmd;

#[derive(Default, Debug, PartialEq)]
pub enum Modality {
    #[default]
    Navigate,
    CellEdit,
    Command,
    // TODO(zaphar): Command Mode?
}

#[derive(Default, Debug)]
pub struct AppState<'ws> {
    pub modality: Modality,
    pub table_state: TableState,
    pub command_state: TextState<'ws>,
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

// Interaction Modalities
// * Navigate
// * Edit
pub struct Workspace<'ws> {
    name: PathBuf,
    book: Book,
    state: AppState<'ws>,
    text_area: TextArea<'ws>,
    dirty: bool,
    show_help: bool,
}

impl<'ws> Workspace<'ws> {
    pub fn new(book: Book, name: PathBuf) -> Self {
        let mut ws = Self {
            book,
            name,
            state: AppState::default(),
            text_area: reset_text_area("".to_owned()),
            dirty: false,
            show_help: false,
        };
        ws.handle_movement_change();
        ws
    }

    pub fn load(path: &PathBuf, locale: &str, tz: &str) -> Result<Self> {
        let book = load_book(path, locale, tz)?;
        Ok(Workspace::new(book, path.clone()))
    }

    pub fn load_into<P: Into<PathBuf>>(&mut self, path: P) -> Result<()> {
        let path: PathBuf = path.into();
        // FIXME(zaphar): This should be managed better.
        let book = load_book(&path, "en", "America/New_York")?;
        self.book = book;
        self.name = path;
        Ok(())
    }

    pub fn move_down(&mut self) -> Result<()> {
        let mut loc = self.book.location.clone();
        let (row_count, _) = self.book.get_size()?;
        if loc.row < row_count {
            loc.row += 1;
            self.book.move_to(loc)?;
        }
        Ok(())
    }

    pub fn move_up(&mut self) -> Result<()> {
        let mut loc = self.book.location.clone();
        if loc.row > 1 {
            loc.row -= 1;
            self.book.move_to(loc)?;
        }
        Ok(())
    }

    pub fn move_left(&mut self) -> Result<()> {
        let mut loc = self.book.location.clone();
        if loc.col > 1 {
            loc.col -= 1;
            self.book.move_to(loc)?;
        }
        Ok(())
    }

    pub fn move_right(&mut self) -> Result<()> {
        let mut loc = self.book.location.clone();
        let (_, col_count) = self.book.get_size()?;
        if loc.col < col_count {
            loc.col += 1;
            self.book.move_to(loc)?;
        }
        Ok(())
    }

    pub fn handle_input(&mut self) -> Result<Option<ExitCode>> {
        if let Event::Key(key) = event::read()? {
            let result = match self.state.modality {
                Modality::Navigate => self.handle_navigation_input(key)?,
                Modality::CellEdit => self.handle_edit_input(key)?,
                Modality::Command => self.handle_command_input(key)?,
            };
            return Ok(result);
        }
        Ok(None)
    }

    fn render_help_text(&self) -> impl Widget {
        let info_block = Block::bordered().title("Help");
        Paragraph::new(match self.state.modality {
            Modality::Navigate => Text::from(vec![
                "Navigate Mode:".into(),
                "* e: Enter edit mode for current cell".into(),
                "* h,j,k,l: vim style navigation".into(),
                "* CTRl-r: Add a row".into(),
                "* CTRl-c: Add a column".into(),
                "* q exit".into(),
                "* Ctrl-S Save sheet".into(),
            ]),
            Modality::CellEdit => Text::from(vec![
                "Edit Mode:".into(),
                "* ESC: Exit edit mode".into(),
                "Otherwise edit as normal".into(),
            ]),
            Modality::Command => Text::from(vec![
                "Command Mode:".into(),
                "* ESC: Exit command mode".into(),
            ]),
        })
        .block(info_block)
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

    fn handle_edit_input(&mut self, key: event::KeyEvent) -> Result<Option<ExitCode>> {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Char('h') if key.modifiers == KeyModifiers::CONTROL => {
                    self.show_help = !self.show_help;
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
            self.dirty = true;
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
                // TODO(jeremy): Modal dialogs?
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
            },
            Ok(Some(Cmd::InsertRow(count))) => {
                self.book.insert_rows(self.book.location.row, count)?;
                self.book.evaluate();
                Ok(true)
            },
            Ok(Some(Cmd::Quit)) => {
                // TODO(zaphar): We probably need to do better than this
                std::process::exit(0);
            },
            Ok(None) => Ok(false),
            Err(_msg) => {
                // TODO(jeremy): Modal dialogs?
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
                KeyCode::Char('h') if key.modifiers == KeyModifiers::CONTROL => {
                    self.show_help = !self.show_help;
                }
                KeyCode::Char('s') if key.modifiers == KeyModifiers::CONTROL => {
                    self.save_file()?;
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
                        self.book.move_to(loc)?;
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
                KeyCode::Char('j') => {
                    self.move_down()?;
                    self.handle_movement_change();
                }
                KeyCode::Char('k') => {
                    self.move_up()?;
                    self.handle_movement_change();
                }
                KeyCode::Char('h') => {
                    self.move_left()?;
                    self.handle_movement_change();
                }
                KeyCode::Char('l') => {
                    self.move_right()?;
                    self.handle_movement_change();
                }
                _ => {
                    // noop
                }
            }
        }
        // TODO(jeremy): Handle some useful navigation operations.
        // * Copy Cell reference
        // * Copy Cell Range reference
        // * Extend Cell {down,up}
        // * Goto location. (Command modality?)
        return Ok(None);
    }

    fn enter_navigation_mode(&mut self) {
        self.state.modality = Modality::Navigate;
    }

    fn enter_command_mode(&mut self) {
        self.state.modality = Modality::Command;
        self.state.command_state.truncate();
        *self.state.command_state.status_mut() = Status::Pending;
        self.state.command_state.focus();
    }

    fn enter_edit_mode(&mut self) {
        self.state.modality = Modality::CellEdit;
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
        self.handle_command(cmd)?;
        self.enter_navigation_mode();
        Ok(())
    }

    fn exit_edit_mode(&mut self) -> Result<()> {
        self.text_area.set_cursor_line_style(Style::default());
        self.text_area.set_cursor_style(Style::default());
        let contents = self.text_area.lines().join("\n");
        if self.dirty {
            self.book.edit_current_cell(contents)?;
            self.book.evaluate();
            self.dirty = false;
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

    fn get_render_parts(
        &mut self,
        area: Rect,
    ) -> Vec<(Rect, Box<dyn Fn(Rect, &mut Buffer, &mut Self)>)> {
        use ratatui::widgets::StatefulWidget;
        let mut cs = vec![Constraint::Fill(4), Constraint::Fill(30)];
        let Address { row, col } = self.book.location;
        let mut rs: Vec<Box<dyn Fn(Rect, &mut Buffer, &mut Self)>> = vec![
            Box::new(|rect: Rect, buf: &mut Buffer, ws: &mut Self| ws.text_area.render(rect, buf)),
            Box::new(move |rect: Rect, buf: &mut Buffer, ws: &mut Self| {
                // Table widget display
                let table_block = Block::bordered();
                let table_inner: Table = TryFrom::try_from(&ws.book).expect("");
                let table = table_inner.block(table_block);
                // https://docs.rs/ratatui/latest/ratatui/widgets/struct.TableState.html
                // TODO(zaphar): Apparently scrolling by columns doesn't work?
                ws.state.table_state.select_cell(Some((row, col)));
                ws.state.table_state.select_column(Some(col));
                StatefulWidget::render(table, rect, buf, &mut ws.state.table_state);
            }),
        ];

        if self.show_help {
            cs.push(Constraint::Fill(9));
            rs.push(Box::new(|rect: Rect, buf: &mut Buffer, ws: &mut Self| {
                let info_para = ws.render_help_text();
                info_para.render(rect, buf);
            }));
        }
        if self.state.modality == Modality::Command {
            cs.push(Constraint::Max(1));
            rs.push(Box::new(|rect: Rect, buf: &mut Buffer, ws: &mut Self| {
                StatefulWidget::render(
                    TextPrompt::from("Command"),
                    rect,
                    buf,
                    &mut ws.state.command_state,
                )
            }));
        }
        let rects: Vec<Rect> = Vec::from(
            Layout::vertical(cs)
                .vertical_margin(2)
                .horizontal_margin(2)
                .flex(Flex::Legacy)
                .split(area.clone())
                .as_ref(),
        );
        rects
            .into_iter()
            .zip(rs.into_iter())
            .map(|(rect, f)| (rect, f))
            .collect()
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

impl<'widget, 'ws: 'widget> Widget for &'widget mut Workspace<'ws> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let outer_block = Block::bordered()
            .title(Line::from(
                self.name
                    .file_name()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| String::from("Unknown")),
            ))
            .title_bottom(match &self.state.modality {
                Modality::Navigate => "navigate",
                Modality::CellEdit => "edit",
                Modality::Command => "command",
            })
            .title_bottom(
                Line::from(format!(
                    "{},{}",
                    self.book.location.row, self.book.location.col
                ))
                .right_aligned(),
            );

        for (rect, f) in self.get_render_parts(area.clone()) {
            f(rect, buf, self);
        }

        outer_block.render(area, buf);
    }
}

const COLNAMES: [&'static str; 26] = [
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S",
    "T", "U", "V", "W", "X", "Y", "Z",
];

impl<'t, 'book: 't> TryFrom<&'book Book> for Table<'t> {
    fn try_from(value: &'book Book) -> std::result::Result<Self, Self::Error> {
        // TODO(zaphar): This is apparently expensive. Maybe we can cache it somehow?
        // We should do the correct thing here if this fails
        let (row_count, col_count) = value.get_size()?;
        let rows: Vec<Row> = (1..=row_count)
            .into_iter()
            .map(|ri| {
                let mut cells = vec![Cell::new(Text::from(ri.to_string()))];
                cells.extend((1..=col_count).into_iter().map(|ci| {
                    // TODO(zaphar): Is this safe?
                    let content = value.get_cell_addr_rendered(ri, ci).unwrap();
                    let cell = Cell::new(Text::raw(content));
                    match (value.location.row == ri, value.location.col == ci) {
                        (true, true) => cell.fg(Color::White).underlined(),
                        _ => cell
                            .bg(if ri % 2 == 0 {
                                Color::Rgb(57, 61, 71)
                            } else {
                                Color::Rgb(165, 169, 160)
                            })
                            .fg(if ri % 2 == 0 {
                                Color::White
                            } else {
                                Color::Rgb(31, 32, 34)
                            }),
                    }
                    .bold()
                }));
                Row::new(cells)
            })
            .collect();
        let mut constraints: Vec<Constraint> = Vec::new();
        constraints.push(Constraint::Max(5));
        for _ in 0..col_count {
            constraints.push(Constraint::Min(5));
        }
        let mut header = Vec::with_capacity(col_count as usize);
        header.push(Cell::new(""));
        header.extend((0..(col_count as usize)).map(|i| {
            let count = (i / 26) + 1;
            Cell::new(COLNAMES[i % 26].repeat(count))
        }));
        Ok(Table::new(rows, constraints)
            .block(Block::bordered())
            .header(Row::new(header).underlined())
            .column_spacing(1)
            .flex(Flex::SpaceAround))
    }

    type Error = anyhow::Error;
}

pub fn draw(frame: &mut Frame, ws: &mut Workspace) {
    frame.render_widget(ws, frame.area());
}
