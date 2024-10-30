use std::{path::PathBuf, process::ExitCode};

use clap::Parser;
use ratatui;
use ui::Workspace;

mod sheet;
mod ui;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg()]
    workbook: PathBuf,
}

fn run(terminal: &mut ratatui::DefaultTerminal, name: PathBuf) -> anyhow::Result<ExitCode> {
    let mut ws = Workspace::load(&name)?;
    loop {
        terminal.draw(|frame| ui::draw(frame, &mut ws))?;
        if let Some(code) = ws.handle_event()? {
            return Ok(code);
        }
    }
}

fn main() -> anyhow::Result<ExitCode> {
    let args = Args::parse();

    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = run(&mut terminal, args.workbook);
    ratatui::restore();
    app_result
}
