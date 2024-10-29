use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    self,
    layout::{Constraint, Layout},
    widgets::{Table, Tabs},
    Frame,
};
use sheet::{Address, CellValue, Tbl};

mod sheet;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg()]
    workbook: PathBuf,
}

fn run(terminal: &mut ratatui::DefaultTerminal) -> std::io::Result<()> {
    loop {
        terminal.draw(|frame| draw(frame))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(());
            }
        }
    }
}

fn generate_default_table<'a>() -> Table<'a> {
    let mut tbl = Tbl::new();
    tbl.update_entry(Address::new(5, 5), CellValue::text("5,5"))
        .context("Failed updating entry at 5,5")
        .unwrap();
    tbl.update_entry(Address::new(10, 10), CellValue::float(10.10))
        .context("Failed updating entry at 10,10")
        .unwrap();
    tbl.update_entry(Address::new(0, 0), CellValue::other("0.0"))
        .context("Failed updating entry at 0,0")
        .unwrap();
    tbl.into()
}

fn draw(frame: &mut Frame) {
    use Constraint::{Min, Percentage};
    let table = generate_default_table();
    let tabs = Tabs::new(vec!["sheet1"]).select(0);
    let rects = Layout::vertical([Min(1), Percentage(90)]).split(frame.area());

    frame.render_widget(tabs, rects[0]);
    frame.render_widget(table, rects[1]);
}

fn main() -> std::io::Result<()> {
    let _ = Args::parse();

    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = run(&mut terminal);
    ratatui::restore();
    app_result
}
