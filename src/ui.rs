use std::io::Stdout;

use crossterm::event::{self, Event};
use ratatui::prelude::CrosstermBackend;

pub mod navigation;

pub fn draw(terminal: &mut ratatui::Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<()> {
    loop {
        terminal.autoresize()?;
        terminal.draw(|frame| {})?;
        if matches!(event::read().expect("failed to read event"), Event::Key(_)) {
            break;
        }
    }
    Ok(())
}
