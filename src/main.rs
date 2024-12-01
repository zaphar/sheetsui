use std::{path::PathBuf, process::ExitCode};

use clap::Parser;
use crossterm::event;
use ratatui;
use serde_json::to_writer;
use std::io::Write;

use ui::Workspace;

mod book;
mod ui;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg()]
    workbook: PathBuf,
    #[arg(default_value_t=String::from("en"), short, long)]
    locale_name: String,
    #[arg(default_value_t=String::from("America/New_York"), short, long)]
    timezone_name: String,
    #[arg(long)]
    log_input: Option<PathBuf>,
}

type ReadFn = Box<dyn FnMut() -> anyhow::Result<event::Event>>;

fn run(terminal: &mut ratatui::DefaultTerminal, args: Args) -> anyhow::Result<ExitCode> {
    let mut ws = Workspace::load(&args.workbook, &args.locale_name, &args.timezone_name)?;
    let mut read_func: ReadFn = if let Some(log_path) = args.log_input {
        {
            let log_file = std::fs::File::create(log_path)?;
            Box::new(move || {
                let evt = event::read()?;
                to_writer(&log_file, &evt)?;
                writeln!(&log_file, "")?;
                Ok(evt)
            })
        }
    } else {
        Box::new(|| {
            let evt = event::read()?;
            Ok(evt)
        })
    };
    loop {
        terminal.draw(|frame| ui::render::draw(frame, &mut ws))?;
        if let Some(code) = ws.handle_input(read_func()?)? {
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
