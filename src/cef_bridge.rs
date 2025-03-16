mod app;
mod client;
mod render;

use crate::{cef_bridge::render::TerminalRenderHandler, utils};
use anyhow::{Result, bail};
use cef::{
    Browser, BrowserSettings, CefStringUtf16, DictionaryValue, Frame, ImplBrowser, ImplFrame,
    RequestContext, browser_host_create_browser_sync, sandbox_info::SandboxInfo,
};
use client::TerminalClient;
use std::{
    env::home_dir,
    sync::{Arc, Mutex},
};
use tracing::{info, trace};

// Structure to hold browser state
#[derive(Debug, Clone)]
pub struct BrowserState {
    pub url: String,
    pub title: String,
    pub loading: bool,
    pub frame_buffer: Arc<Mutex<Vec<u8>>>,
    pub frame_dimensions: (i32, i32),
}

impl Default for BrowserState {
    fn default() -> Self {
        Self {
            url: String::from("about:blank"),
            title: String::new(),
            loading: false,
            frame_buffer: Arc::new(Mutex::new(Vec::new())),
            frame_dimensions: (1280, 720), // 设置默认尺寸避免初始化时为0
        }
    }
}

// Main browser client implementation
#[derive(Clone)]
pub struct CarbideClient {
    state: Arc<Mutex<BrowserState>>,
    // render_handler: Arc<TerminalRenderHandler>,
    // window_info: cef::WindowInfo,
    // client: cef::Client,
    browser_host: cef::Browser,
}

impl CarbideClient {
    /// Initializes a new instance of `CarbideClient`.
    ///
    /// This function sets up the necessary components for the CEF-based browser client,
    /// including the sandbox, application, and browser settings. It initializes the CEF
    /// library, creates the browser state, render handler, and client. If initialization
    /// fails, it returns an error.
    ///
    /// # Returns
    /// An `anyhow::Result` containing the `CarbideClient` instance if successful, or an error
    /// if initialization fails.
    ///
    /// # Safety
    /// must be called from the main thread
    pub fn new() -> anyhow::Result<Self> {
        info!("Creating CarbideClient");

        let state = Arc::new(Mutex::new(BrowserState::default()));
        let mut window_info = cef::WindowInfo {
            windowless_rendering_enabled: true.into(),
            ..Default::default()
        };
        let render_handler = TerminalRenderHandler::new(state.clone());
        trace!("created render handler");

        let mut client = TerminalClient::new(render_handler.clone());
        trace!("created client");

        let browser_settings = BrowserSettings {
            windowless_frame_rate: 60,
            ..Default::default()
        };
        let browser = match browser_host_create_browser_sync(
            Some(&window_info),
            Some(&mut client),
            Some(&CefStringUtf16::from("about:blank")),
            Some(&browser_settings),
            Option::<&mut DictionaryValue>::None,
            Option::<&mut RequestContext>::None,
        ) {
            Some(browser) => browser,
            None => {
                bail!("Failed to create browser");
            }
        };
        trace!("created browser");
        
        Ok(Self {
            state,
            // render_handler,
            // window_info,
            // client,
            browser_host: browser,
        })
    }

    pub fn get_frame_data(&self) -> Option<(Vec<u8>, (i32, i32))> {
        let state = self.state.lock().ok()?;
        let buffer = state.frame_buffer.lock().ok()?;
        Some((buffer.clone(), state.frame_dimensions))
    }

    pub fn navigate(&self, url: &str) -> Result<()> {
        if let Some(browser) = self.get_browser() {
            if let Some(x) = browser.get_main_frame() {
                x.load_url(Some(&CefStringUtf16::from(url)))
            }
        }
        Ok(())
    }

    pub fn get_browser(&self) -> Option<Browser> {
        Some(self.browser_host.clone())
    }
}

// Implement required traits for CarbideClient
impl CarbideClient {
    fn on_title_change(&self, browser: Browser, title: &str) {
        if let Ok(mut state) = self.state.lock() {
            state.title = title.to_string();
        }
    }

    fn on_loading_state_change(&self, browser: Browser, loading: bool) {
        if let Ok(mut state) = self.state.lock() {
            state.loading = loading;
        }
    }

    fn on_address_change(&self, browser: Browser, frame: Frame, url: &str) {
        if frame.is_main() != 0 {
            if let Ok(mut state) = self.state.lock() {
                state.url = url.to_string();
            }
        }
    }
}

// Shutdown CEF
pub fn shutdown_cef() -> Result<()> {
    cef::shutdown();
    Ok(())
}

pub fn init_cef() -> Result<()> {
    info!("Initializing CEF");
    let args = cef::args::Args::new();
    let cmd = args.as_cmd_line().unwrap();

    let sandbox = SandboxInfo::new();
    let opts = cef::Settings {
        windowless_rendering_enabled: true.into(),
        // TODO: sandbox
        no_sandbox: true.into(),
        log_file: CefStringUtf16::from(
            std::path::PathBuf::from(utils::log::LOG_OUTPUT_DIR)
                .join("cef.log")
                .to_str()
                .unwrap(),
        ),
        root_cache_path: home_dir()
            .unwrap()
            .join("carbide/cache")
            .to_str()
            .unwrap()
            .into(),
        ..Default::default()
    };

    let mut app = app::TerminalApp::new();
    info!("created app");
    // let code = execute_process(None, Some(&mut app), sandbox.as_mut_ptr());
    // if code != 0 {
    //     tracing::error!("CEF process execution failed with code: {}", code);
    //     bail!("cef process execution failed, code {}", code)
    // }

    let code = cef::initialize(
        Some(args.as_main_args()),
        Some(&opts),
        Some(&mut app),
        sandbox.as_mut_ptr(),
    );
    if code == 0 {
        tracing::error!("CEF initialization failed with code: {}", code);
        bail!("cef init failed, code {}", code)
    }
    info!("initialized cef app instance");
    Ok(())
}
