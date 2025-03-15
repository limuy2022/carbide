use anyhow::{Result, bail};
use cef::{
    App, Browser, BrowserHost, BrowserSettings, CefStringUtf16, Client, DictionaryValue, Frame,
    ImplBrowser, ImplFrame, ImplRenderHandler, PaintElementType, ProcessMessage, Rect,
    RenderProcessHandler, RequestContext, Value, WrapRenderHandler,
    browser_host_create_browser_sync,
    rc::{Rc, RcImpl},
};
use std::{
    ffi::c_int,
    ptr,
    sync::{Arc, Mutex},
};

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
            frame_dimensions: (0, 0),
        }
    }
}

// Main browser client implementation
#[derive(Clone)]
pub struct CarbideClient {
    state: Arc<Mutex<BrowserState>>,
    render_handler: Arc<TerminalRenderHandler>,
}

impl CarbideClient {
    pub fn new() -> Self {
        let state = Arc::new(Mutex::new(BrowserState::default()));
        Self {
            state: state.clone(),
            render_handler: Arc::new(TerminalRenderHandler::new(state)),
        }
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
        let window_info = cef::WindowInfo {
            windowless_rendering_enabled: true.into(),
            ..Default::default()
        };

        // let client = cef::Client::new(self.render_handler.clone() as Arc<dyn cef::RenderHandler>);
        let mut client = cef::Client::default();

        browser_host_create_browser_sync(
            Some(&window_info),
            Some(&mut client),
            Some(&CefStringUtf16::from("about:blank")),
            Some(&BrowserSettings::default()),
            Option::<&mut DictionaryValue>::None,
            Option::<&mut RequestContext>::None,
        )
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

#[derive(Debug)]
struct TerminalRenderHandler {
    state: Arc<Mutex<BrowserState>>,
    object: *mut RcImpl<cef_dll_sys::_cef_render_handler_t, Self>,
}

impl TerminalRenderHandler {
    fn new(state: Arc<Mutex<BrowserState>>) -> Self {
        Self {
            state,
            object: ptr::null_mut(),
        }
    }
}

impl Clone for TerminalRenderHandler {
    fn clone(&self) -> Self {
        let object = unsafe {
            let rc_impl = &mut *self.object;
            rc_impl.interface.add_ref();
            self.object
        };
        let state = self.state.clone();
        Self { state, object }
    }
}

impl cef::rc::Rc for TerminalRenderHandler {
    fn as_base(&self) -> &cef_dll_sys::cef_base_ref_counted_t {
        unsafe {
            let rc_impl = &*self.object;
            &rc_impl.cef_object.base
        }
    }
}

impl WrapRenderHandler for TerminalRenderHandler {
    fn wrap_rc(&mut self, object: *mut RcImpl<cef_dll_sys::_cef_render_handler_t, Self>) {
        self.object = object;
    }
}

impl ImplRenderHandler for TerminalRenderHandler {
    fn get_raw(&self) -> *mut cef_dll_sys::_cef_render_handler_t {
        self.object as *mut _
    }

    fn get_view_rect(&self, browser: Option<&mut impl ImplBrowser>, rect: Option<&mut Rect>) {
        let state = self.state.lock().unwrap();
        if let Some(rect) = rect {
            *rect = Rect {
                x: 0,
                y: 0,
                width: state.frame_dimensions.0,
                height: state.frame_dimensions.1,
            };
        }
    }

    fn on_paint(
        &self,
        browser: Option<&mut impl ImplBrowser>,
        type_: PaintElementType,
        dirty_rects_count: usize,
        dirty_rects: Option<&Rect>,
        buffer: *const u8,
        width: c_int,
        height: c_int,
    ) {
        if *type_.as_ref() == cef_dll_sys::cef_paint_element_type_t::PET_VIEW {
            let mut state: std::sync::MutexGuard<'_, BrowserState> = self.state.lock().unwrap();
            state.frame_dimensions = (width, height);
            let buffer_slice =
                unsafe { std::slice::from_raw_parts(buffer, (width * height * 4) as usize) };
            let mut frame_buf = state.frame_buffer.lock().unwrap();
            frame_buf.resize(buffer_slice.len(), 0);
            frame_buf.copy_from_slice(buffer_slice);
        }
    }
}

// Initialize CEF
pub fn initialize_cef() -> anyhow::Result<()> {
    let opts = cef::Settings {
        windowless_rendering_enabled: true.into(),
        ..Default::default()
    };
    let code = cef::initialize(None, Some(&opts), Option::<&mut App>::None, ptr::null_mut());
    if code != 0 {
        bail!("cef init failed, code {}", code)
    }
    Ok(())
}

// Shutdown CEF
pub fn shutdown_cef() -> Result<()> {
    cef::shutdown();
    Ok(())
}
