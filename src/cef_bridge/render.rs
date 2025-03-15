use crate::cef_bridge::BrowserState;
use cef::rc::{Rc, RcImpl};
use cef::{ImplBrowser, ImplRenderHandler, PaintElementType, Rect, WrapRenderHandler};
use std::ffi::c_int;
use std::ptr;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct TerminalRenderHandler {
    state: Arc<Mutex<BrowserState>>,
    object: *mut RcImpl<cef_dll_sys::_cef_render_handler_t, Self>,
}

impl TerminalRenderHandler {
    pub fn new(state: Arc<Mutex<BrowserState>>) -> Self {
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
            // Convert BGRA to RGB and flip vertically
            let mut rgb_buf = Vec::with_capacity((width * height * 3) as usize);
            for y in (0..height).rev() {
                for x in 0..width {
                    let idx = (y * width + x) as usize * 4;
                    rgb_buf.push(buffer_slice[idx + 2]); // R
                    rgb_buf.push(buffer_slice[idx + 1]); // G 
                    rgb_buf.push(buffer_slice[idx]); // B
                }
            }
            frame_buf.resize(rgb_buf.len(), 0);
            frame_buf.copy_from_slice(&rgb_buf);
        }
    }
}
