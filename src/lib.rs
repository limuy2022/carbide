pub mod cli;
pub mod gfx;
pub mod input;
pub mod output;
pub mod ui;

mod utils;

use std::io::stdout;

use clap::Parser;
use cli::CommandLine;
use shadow_rs::shadow;

shadow!(build);

pub fn run() -> anyhow::Result<()> {
    utils::log::logger_init(stdout, build::PROJECT_NAME);
    let parser = CommandLine::parse();
    if parser.version {
        println!("{}", build::VERSION);
        return Ok(());
    }
    let mut terminal = ratatui::init();
    ui::draw(&mut terminal)?;
    ratatui::restore();
    Ok(())
}
