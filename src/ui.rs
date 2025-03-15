use std::io::Stdout;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

use crate::cef_bridge::{self, BrowserState, CarbideClient};

pub mod navigation;

pub struct BrowserUI {
    client: CarbideClient,
    address: String,
}

impl BrowserUI {
    pub fn new() -> Self {
        Self {
            client: CarbideClient::new(),
            address: String::new(),
        }
    }

    pub fn navigate(&self, url: &str) -> anyhow::Result<()> {
        self.client.navigate(url)
    }

    fn get_pixel(buffer: &[u8], width: usize, x: usize, y: usize) -> (u8, u8, u8) {
        let x = x.min(width - 1);
        let y = y.min(buffer.len() / (width * 4) - 1);
        let idx = (y * width + x) * 4;
        (
            *buffer.get(idx).unwrap_or(&0),
            *buffer.get(idx + 1).unwrap_or(&0),
            *buffer.get(idx + 2).unwrap_or(&0),
        )
    }

    fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
        if r == g && g == b {
            let gray = (r as f32 * 0.2126 + g as f32 * 0.7152 + b as f32 * 0.0722) / 12.75;
            return 232 + gray.round() as u8;
        }
        
        16 + 
        ((r as u16 * 5 / 255) as u8 * 36) + 
        ((g as u16 * 5 / 255) as u8 * 6) + 
        (b as u16 * 5 / 255) as u8
    }

    fn convert_frame_to_ansi(buffer: &[u8], width: usize, height: usize) -> String {
        let mut output = String::with_capacity(width * height * 20);
        let mut i = 0;
        
        // 使用每2x2像素块渲染为一个字符
        for y in (0..height).step_by(2) {
            for x in (0..width).step_by(2) {
                // 获取四个像素点
                let pixels = [
                    Self::get_pixel(buffer, width, x, y),
                    Self::get_pixel(buffer, width, x+1, y),
                    Self::get_pixel(buffer, width, x, y+1),
                    Self::get_pixel(buffer, width, x+1, y+1),
                ];
                
                // 计算平均颜色
                let (r, g, b) = (
                    pixels.iter().map(|p| p.0 as u32).sum::<u32>() / 4,
                    pixels.iter().map(|p| p.1 as u32).sum::<u32>() / 4,
                    pixels.iter().map(|p| p.2 as u32).sum::<u32>() / 4,
                );
                
                // 转换为ANSI 256色
                let color = Self::rgb_to_ansi256(r as u8, g as u8, b as u8);
                
                // 使用Unicode上半块字符
                output.push_str(&format!(
                    "\x1b[48;5;{}m▀",
                    color
                ));
            }
            output.push_str("\x1b[0m\n");
        }
        output
    }
}

pub fn draw(terminal: &mut ratatui::Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<()> {
    let mut browser_ui = BrowserUI::new();
    let mut should_quit = false;

    // Initialize CEF
    cef_bridge::initialize_cef()?;

    while !should_quit {
        terminal.autoresize()?;
        terminal.draw(|frame| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Browser bar
                    Constraint::Min(0),    // Content area
                ])
                .split(frame.area());

            // Browser bar layout
            let browser_bar = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(3), // Back button
                    Constraint::Length(3), // Forward button
                    Constraint::Min(0),    // Address bar
                    Constraint::Length(3), // Refresh button
                ])
                .split(layout[0]);

            // Draw navigation buttons
            let back_btn = Paragraph::new("←")
                .block(Block::default().borders(Borders::ALL))
                .alignment(Alignment::Center);
            frame.render_widget(back_btn, browser_bar[0]);

            let forward_btn = Paragraph::new("→")
                .block(Block::default().borders(Borders::ALL))
                .alignment(Alignment::Center);
            frame.render_widget(forward_btn, browser_bar[1]);

            // Draw address bar
            let address_bar = Paragraph::new(browser_ui.address.as_str())
                .block(Block::default().borders(Borders::ALL).title("Address"))
                .alignment(Alignment::Left);
            frame.render_widget(address_bar, browser_bar[2]);

            let refresh_btn = Paragraph::new("↻")
                .block(Block::default().borders(Borders::ALL))
                .alignment(Alignment::Center);
            frame.render_widget(refresh_btn, browser_bar[3]);

            // Draw CEF content
            let content_text = match browser_ui.client.get_frame_data() {
                Some((buffer, (width, height))) if !buffer.is_empty() => {
                    BrowserUI::convert_frame_to_ansi(&buffer, width as usize, height as usize)
                }
                _ => String::from("Loading..."),
            };
            
            let content = Paragraph::new(content_text)
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(content, layout[1]);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char(c) => browser_ui.address.push(c),
                        KeyCode::Backspace => {
                            browser_ui.address.pop();
                        }
                        KeyCode::Enter => {
                            if !browser_ui.address.is_empty() {
                                if let Err(e) = browser_ui.navigate(&browser_ui.address) {
                                    eprintln!("Navigation error: {}", e);
                                }
                            }
                        }
                        KeyCode::Esc => should_quit = true,
                        _ => {}
                    }
                }
            }
        }
    }

    // Shutdown CEF
    cef_bridge::shutdown_cef()?;
    Ok(())
}
