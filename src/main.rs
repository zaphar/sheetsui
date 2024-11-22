use std::{path::PathBuf, process::ExitCode};

use clap::Parser;
use crossterm::event;
use ratatui;
use ui::Workspace;

mod ui;
mod book;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg()]
    workbook: PathBuf,
    #[arg(default_value_t=String::from("en"), short, long)]
    locale_name: String,
    #[arg(default_value_t=String::from("America/New_York"), short, long)]
    timezone_name: String,
}

fn run(terminal: &mut ratatui::DefaultTerminal, args: Args) -> anyhow::Result<ExitCode> {
    let mut ws = Workspace::load(&args.workbook, &args.locale_name, &args.timezone_name)?;

    loop {
        terminal.draw(|frame| ui::render::draw(frame, &mut ws))?;
        if let Some(code) = ws.handle_input(event::read()?)? {
            return Ok(code);
        }
    }
}

fn main() -> anyhow::Result<ExitCode> {
    let args = Args::parse();

    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = run(&mut terminal, args);
    ratatui::restore();
    app_result
}
