//! Ui rendering logic

use std::{path::PathBuf, process::ExitCode};

use crate::book::Book;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ironcalc::base::worksheet::WorksheetDimension;
use ratatui::{
    self,
    layout::{Constraint, Flex, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Cell, Paragraph, Row, Table, TableState, Widget},
    Frame,
};
use tui_textarea::{CursorMove, TextArea};

#[derive(Default, Debug, PartialEq)]
pub enum Modality {
    #[default]
    Navigate,
    CellEdit,
    // TODO(zaphar): Command Mode?
}

#[derive(Default, Debug)]
pub struct AppState {
    pub modality: Modality,
    pub table_state: TableState,
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
    state: AppState,
    text_area: TextArea<'ws>,
    dirty: bool,
}

impl<'ws> Workspace<'ws> {
    pub fn new(book: Book, name: PathBuf) -> Self {
        let mut ws = Self {
            book,
            name,
            state: AppState::default(),
            text_area: reset_text_area("".to_owned()),
            dirty: false,
        };
        ws.handle_movement_change();
        ws
    }

    pub fn load(path: &PathBuf, locale: &str, tz: &str) -> Result<Self> {
        let book = if path.exists() {
            Book::new_from_xlsx_with_locale(&path.to_string_lossy().to_string(), locale, tz)?
        } else {
            Book::default()
        };
        //book.move_to(Address { row: 0, col: 0 })?;
        Ok(Workspace::new(book, path.clone()))
    }

    pub fn move_down(&mut self) -> Result<()> {
        let mut loc = self.book.location.clone();
        let WorksheetDimension { min_row: _, max_row, min_column: _, max_column: _ } = self.book.get_dimensions()?;
        if loc.row <= max_row as usize {
            loc.row += 1;
            self.book.move_to(loc)?;
        }
        Ok(())
    }

    pub fn move_up(&mut self) -> Result<()> {
        let mut loc = self.book.location.clone();
        let WorksheetDimension { min_row, max_row: _, min_column: _, max_column: _ } = self.book.get_dimensions()?;
        if loc.row > min_row as usize {
            loc.row -= 1;
            self.book.move_to(loc)?;
        }
        Ok(())
    }

    pub fn move_left(&mut self) -> Result<()> {
        let mut loc = self.book.location.clone();
        let WorksheetDimension { min_row: _, max_row: _, min_column, max_column: _ } = self.book.get_dimensions()?;
        if loc.col > min_column as usize {
            loc.col -= 1;
            self.book.move_to(loc)?;
        }
        Ok(())
    }

    pub fn move_right(&mut self) -> Result<()> {
        let mut loc = self.book.location.clone();
        let WorksheetDimension { min_row: _, max_row: _, min_column: _, max_column} = self.book.get_dimensions()?;
        if loc.col < max_column as usize {
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
        })
        .block(info_block)
    }

    fn handle_edit_input(&mut self, key: event::KeyEvent) -> Result<Option<ExitCode>> {
        if key.kind == KeyEventKind::Press {
            if let KeyCode::Esc = key.code {
                self.state.modality = Modality::Navigate;
                self.text_area.set_cursor_line_style(Style::default());
                self.text_area.set_cursor_style(Style::default());
                let contents = self.text_area.lines().join("\n");
                if self.dirty {
                    self.book.edit_current_cell(contents)?;
                }
                return Ok(None);
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

    fn handle_navigation_input(&mut self, key: event::KeyEvent) -> Result<Option<ExitCode>> {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Char('e') => {
                    self.state.modality = Modality::CellEdit;
                    self.text_area
                        .set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
                    self.text_area
                        .set_cursor_style(Style::default().add_modifier(Modifier::SLOW_BLINK));
                    self.text_area.move_cursor(CursorMove::Bottom);
                    self.text_area.move_cursor(CursorMove::End);
                }
                KeyCode::Char('s') if key.modifiers == KeyModifiers::CONTROL => {
                    self.save_file()?;
                }
                KeyCode::Char('r') if key.modifiers == KeyModifiers::CONTROL => {
                    let (row_count, _) = self.book.get_size()?;
                    self.book.update_entry(&Address {row: row_count+1, col: 1 }, "")?;
                    let (row, _) = self.book.get_size()?;
                    let mut loc = self.book.location.clone();
                    if loc.row < row as usize {
                        loc.row = row as usize;
                        self.book.move_to(loc)?;
                    }
                    self.handle_movement_change();
                }
                KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                    let (_, col_count) = self.book.get_size()?;
                    self.book.update_entry(&Address {row: 1, col: col_count+1 }, "")?;
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

    fn handle_movement_change(&mut self) {
        let contents = self
            .book
            .get_current_cell_contents()
            .expect("Unexpected failure getting current cell contents");
        self.text_area = reset_text_area(contents);
    }

    fn save_file(&self) -> Result<()> {
        self.book.save_to_xlsx(&self.name.to_string_lossy().to_string())?;
        Ok(())
    }
}

fn reset_text_area<'a>(content: String) -> TextArea<'a> {
    let mut text_area = TextArea::from(content.lines());
    text_area.set_cursor_line_style(Style::default());
    text_area.set_cursor_style(Style::default());
    text_area
}

impl<'widget, 'ws: 'widget> Widget for &'widget mut Workspace<'ws> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
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
            })
            .title_bottom(
                Line::from(format!(
                    "{},{}",
                    self.book.location.row, self.book.location.col
                ))
                .right_aligned(),
            );
        let [edit_rect, table_rect, info_rect] = Layout::vertical(&[
            Constraint::Fill(1),
            Constraint::Fill(30),
            Constraint::Fill(9),
        ])
        .vertical_margin(2)
        .horizontal_margin(2)
        .flex(Flex::Legacy)
        .areas(area.clone());
        outer_block.render(area, buf);
        self.text_area.render(edit_rect, buf);
        let table_block = Block::bordered();
        let table_inner: Table = TryFrom::try_from(&self.book).expect("");
        let table = table_inner.block(table_block);
        // https://docs.rs/ratatui/latest/ratatui/widgets/struct.TableState.html
        let Address { row, col } = self.book.location;
        // TODO(zaphar): Apparently scrolling by columns doesn't work?
        self.state.table_state.select_cell(Some((row, col)));
        self.state.table_state.select_column(Some(col));
        use ratatui::widgets::StatefulWidget;
        StatefulWidget::render(table, table_rect, buf, &mut self.state.table_state);
        //table.render_stateful(table_rect, buf);
        let info_para = self.render_help_text();
        info_para.render(info_rect, buf);
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
                cells.extend((1..=col_count)
                    .into_iter()
                    .map(|ci| {
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
