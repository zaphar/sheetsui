use std::path::PathBuf;

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui;

mod sheet;
mod ui;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg()]
    workbook: PathBuf,
}

fn run(terminal: &mut ratatui::DefaultTerminal, name: PathBuf) -> std::io::Result<()> {
    loop {
        terminal.draw(|frame| ui::draw(frame, &name))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(());
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = run(&mut terminal, args.workbook);
    ratatui::restore();
    app_result
}
