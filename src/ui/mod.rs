//! Ui rendering logic

use std::{
    fs::File,
    io::Read,
    path::PathBuf,
    process::ExitCode,
};

use super::sheet::{Address, Tbl};

use anyhow::{anyhow, Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    self,
    layout::{Constraint, Flex, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Cell, Paragraph, Row, Table, Widget},
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
}

// Interaction Modalities
// * Navigate
// * Edit
pub struct Workspace<'ws> {
    name: PathBuf,
    tbl: Tbl,
    state: AppState,
    text_area: TextArea<'ws>,
    dirty: bool,
}

impl<'ws> Workspace<'ws> {
    pub fn new(tbl: Tbl, name: PathBuf) -> Self {
        let mut ws = Self {
            tbl,
            name,
            state: AppState::default(),
            text_area: reset_text_area("".to_owned()),
            dirty: false,
        };
        ws.handle_movement_change();
        ws
    }

    pub fn load(path: &PathBuf) -> Result<Self> {
        let input = if path.exists() {
            if path.is_file() {
                let mut f = File::open(path)?;
                let mut buf = Vec::new();
                let _ = f.read_to_end(&mut buf)?;
                String::from_utf8(buf).context(format!("Error reading file: {:?}", path))?
            } else {
                return Err(anyhow!("Not a valid path: {}", path.to_string_lossy().to_string()));
            }
        } else {
            String::from(",,,\n,,,\n")
        };
        let mut tbl = Tbl::from_str(input)?;
        tbl.move_to(Address { row: 0, col: 0 })?;
        Ok(Workspace::new(
            tbl,
            path.clone(),
        ))
    }

    pub fn move_down(&mut self) -> Result<()> {
        let mut loc = self.tbl.location.clone();
        let (row, _) = self.tbl.dimensions();
        if loc.row < row - 1 {
            loc.row += 1;
            self.tbl.move_to(loc)?;
        }
        Ok(())
    }

    pub fn move_up(&mut self) -> Result<()> {
        let mut loc = self.tbl.location.clone();
        if loc.row > 0 {
            loc.row -= 1;
            self.tbl.move_to(loc)?;
        }
        Ok(())
    }

    pub fn move_left(&mut self) -> Result<()> {
        let mut loc = self.tbl.location.clone();
        if loc.col > 0 {
            loc.col -= 1;
            self.tbl.move_to(loc)?;
        }
        Ok(())
    }

    pub fn move_right(&mut self) -> Result<()> {
        let mut loc = self.tbl.location.clone();
        let (_, col) = self.tbl.dimensions();
        if loc.col < col - 1 {
            loc.col += 1;
            self.tbl.move_to(loc)?;
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
                "* q exit\n* Ctrl-S Save sheet".into(),
            ]),
            Modality::CellEdit => Text::from(vec![
                "Edit Mode:".into(),
                "* ESC: Exit edit mode".into(),
                "Otherwise edit as normal".into(),
            ]),
        }).block(info_block)
    }

    fn handle_edit_input(&mut self, key: event::KeyEvent) -> Result<Option<ExitCode>> {
        if key.kind == KeyEventKind::Press {
            if let KeyCode::Esc = key.code {
                self.state.modality = Modality::Navigate;
                self.text_area.set_cursor_line_style(Style::default());
                self.text_area.set_cursor_style(Style::default());
                let contents = self.text_area.lines().join("\n");
                if self.dirty {
                    let loc = self.tbl.location.clone();
                    self.tbl.update_entry(&loc, contents)?;
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
        // * Insert row
        // * Insert column
        return Ok(None);
    }

    fn handle_movement_change(&mut self) {
        let contents = self.tbl.get_raw_value(&self.tbl.location);
        self.text_area = reset_text_area(contents);
    }

    fn save_file(&self) -> Result<()> {
        let contents = self.tbl.csv.export_raw_table().map_err(|e| anyhow::anyhow!("Error serializing to csv: {:?}", e))?;
        std::fs::write(&self.name, contents)?;
        Ok(())
    }
}

fn reset_text_area<'a>(content: String) -> TextArea<'a> {
    let mut text_area = TextArea::from(content.lines());
    text_area.set_cursor_line_style(Style::default());
    text_area.set_cursor_style(Style::default());
    text_area
}

impl<'widget, 'ws: 'widget> Widget for &'widget Workspace<'ws> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let outer_block = Block::bordered()
            .title(Line::from(self.name
                .file_name().map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| String::from("Unknown"))))
            .title_bottom(match &self.state.modality {
                Modality::Navigate => "navigate",
                Modality::CellEdit => "edit",
            })
            .title_bottom(
                Line::from(format!(
                    "{},{}",
                    self.tbl.location.row, self.tbl.location.col
                ))
                .right_aligned(),
            );
        let [edit_rect, table_rect, info_rect] =
            Layout::vertical(&[Constraint::Fill(1), Constraint::Fill(20), Constraint::Fill(3)])
                .vertical_margin(2)
                .horizontal_margin(2)
                .flex(Flex::Legacy)
                .areas(area.clone());
        outer_block.render(area, buf);
        self.text_area.render(edit_rect, buf);
        let table_block = Block::bordered();
        let table = Table::from(&self.tbl).block(table_block);
        table.render(table_rect, buf);
        // TODO(jwall): render help text?
        let info_para = self.render_help_text();
        info_para.render(info_rect, buf);
    }
}

const COLNAMES: [&'static str; 27] = [
    "", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R",
    "S", "T", "U", "V", "W", "X", "Y", "Z",
];

impl<'t> From<&Tbl> for Table<'t> {
    fn from(value: &Tbl) -> Self {
        let (_, cols) = value.dimensions();
        let rows: Vec<Row> = value
            .csv
            .get_calculated_table()
            .iter()
            .enumerate()
            .map(|(ri, r)| {
                let cells =
                    vec![Cell::new(format!("{}", ri))]
                        .into_iter()
                        .chain(r.iter().enumerate().map(|(ci, v)| {
                            let content = format!("{}", v);
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
        // TODO(zaphar): Handle the double letter column names
        let header: Vec<Cell> = (0..=cols).map(|i| Cell::new(COLNAMES[i % 26])).collect();
        let mut constraints: Vec<Constraint> = Vec::new();
        constraints.push(Constraint::Max(5));
        for _ in 0..cols {
            constraints.push(Constraint::Min(5));
        }
        Table::new(rows, constraints)
            .block(Block::bordered())
            .header(Row::new(header).underlined())
            .column_spacing(1)
            .flex(Flex::SpaceAround)
    }
}

pub fn draw(frame: &mut Frame, ws: &Workspace) {
    frame.render_widget(ws, frame.area());
}
