use std::path::PathBuf;

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    self,
    layout::{Constraint, Layout},
    widgets::{Cell, Row, Table, Tabs},
    Frame,
};

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

fn generate_default_rows<'a>(row_count: usize, col_count: usize) -> Vec<Row<'a>> {
    (0..row_count)
        .map(|ri| {
            (0..col_count)
                .map(|ci| Cell::new(format!(" {}:{} ", ri, ci)))
                .collect::<Row>()
                .height(2)
        })
        .collect::<Vec<Row>>()
}

fn draw(frame: &mut Frame) {
    use Constraint::{Min, Percentage};
    let table = Table::default().rows(generate_default_rows(10, 10));
    let tabs = Tabs::new(vec!["sheet1"]).select(0);
    let rects = Layout::vertical([Min(1), Percentage(90)]).split(frame.area());

    frame.render_widget(tabs, rects[0]);
    frame.render_widget(table, rects[1]);
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    
    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = run(&mut terminal);
    ratatui::restore();
    app_result
}
