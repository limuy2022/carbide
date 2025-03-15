pub mod cef_bridge;
pub mod cli;
pub mod gfx;
pub mod input;
pub mod output;
pub mod ui;

mod utils;

use cef::{api_hash, run_message_loop};
use clap::Parser;
use cli::CommandLine;
use parking_lot::Mutex;
use shadow_rs::shadow;
use std::{sync::Arc, thread::sleep, time};
use tracing::info;

shadow!(build);

pub fn run() -> anyhow::Result<()> {
    utils::log::logger_init(build::PROJECT_NAME);
    let _ = api_hash(cef_dll_sys::CEF_API_VERSION_LAST, 0);

    let parser = CommandLine::parse();
    if parser.version {
        println!("{}", build::VERSION);
        return Ok(());
    }
    // create a thread to draw the UI
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let barrier_clone = barrier.clone();
    let cef_status = std::sync::Arc::new(Mutex::new(false));
    let cef_status_clone = cef_status.clone();
    cef_bridge::init_cef()?;
    // create the browser
    let browser = Arc::new(match cef_bridge::CarbideClient::new() {
        Ok(browser) => browser,
        Err(e) => {
            barrier_clone.wait();
            return Err(e);
        }
    });
    let handle = std::thread::spawn(move || {
        info!("Browser created");
        *cef_status_clone.lock() = true;
        barrier_clone.wait();
        info!("Starting UI");
        let mut terminal = ratatui::init();
        sleep(time::Duration::from_millis(500));
        ui::draw(&mut terminal, browser)?;
        ratatui::restore();
        anyhow::Ok(())
    });

    barrier.wait();
    if *cef_status.lock() {
        info!("Starting CEF message loop");
        run_message_loop();
    }
    handle.join().unwrap()?;
    Ok(())
}
