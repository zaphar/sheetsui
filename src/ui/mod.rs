//! Ui rendering logic

use std::{fs::File, io::Read, path::PathBuf};

use super::sheet::{Address, CellValue, Tbl};

use anyhow::{Context, Result};
use ratatui::{
    self,
    layout::{Constraint, Flex, Layout},
    style::{Color, Stylize},
    text::Text,
    widgets::{Block, Cell, Row, Table, Tabs, Widget},
    Frame,
};

pub struct Workspace {
    name: String,
    tbl: Tbl,
}

impl Workspace {
    pub fn new<S: Into<String>>(tbl: Tbl, name: S) -> Self {
        Self {
            tbl,
            name: name.into(),
        }
    }

    pub fn load(path: &PathBuf) -> Result<Self> {
        let mut f = File::open(path)?;
        let mut buf = Vec::new();
        let _ = f.read_to_end(&mut buf)?;
        let input = String::from_utf8(buf)
            .context(format!("Error reading file: {:?}", path))?;
        let tbl = Tbl::from_str(input)?;
        Ok(Workspace::new(tbl, path.file_name().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| "Unknown".to_string())))
    }
}

impl Widget for Workspace {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        use Constraint::{Min, Percentage};
        let rects = Layout::vertical([Min(1), Percentage(90)]).split(area);
        let table = Table::from(&self.tbl);
        let tabs = Tabs::new(vec![self.name.clone()]).select(0);

        tabs.render(rects[0], buf);
        table.render(rects[1], buf);
    }
}

fn generate_default_table<'a>() -> Tbl {
    let mut tbl = Tbl::new();
    tbl.update_entry(Address::new(3, 3), CellValue::text("3,3"))
        .context("Failed updating entry at 5,5")
        .expect("Unexpected fail to update entry");
    tbl.update_entry(Address::new(6, 6), CellValue::float(6.6))
        .context("Failed updating entry at 10,10")
        .expect("Unexpected fail to update entry");
    tbl.update_entry(Address::new(0, 0), CellValue::formula("0.0"))
        .context("Failed updating entry at 0,0")
        .expect("Unexpected fail to update entry");
    tbl.update_entry(Address::new(1, 0), CellValue::formula("1.0"))
        .context("Failed updating entry at 0,0")
        .expect("Unexpected fail to update entry");
    tbl.update_entry(Address::new(2, 0), CellValue::formula("2.0"))
        .context("Failed updating entry at 0,0")
        .expect("Unexpected fail to update entry");
    tbl
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
            .map(|(i, r)| {
                let cells = vec![Cell::new(format!("{}", i))]
                    .into_iter()
                    .chain(r.iter().map(|v| {
                        let content = format!("{}", v);
                        Cell::new(Text::raw(content))
                            .bg(if i % 2 == 0 {
                                Color::Rgb(57, 61, 71)
                            } else {
                                Color::Rgb(165, 169, 160)
                            })
                            .fg(if i % 2 == 0 {
                                Color::White
                            } else {
                                Color::Rgb(31,32,34)
                            })
                            .underlined()
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

pub fn draw(frame: &mut Frame, name: &PathBuf) {
    let table = generate_default_table();
    let ws = Workspace::load(name).unwrap();

    frame.render_widget(ws, frame.area());
}
