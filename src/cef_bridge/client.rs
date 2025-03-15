use crate::cef_bridge::TerminalRenderHandler;
use cef::rc::{Rc, RcImpl};
use cef::{Client, ImplClient, RenderHandler};
use std::ptr;
use std::sync::Arc;

pub struct TerminalClient {
    object: *mut RcImpl<cef_dll_sys::cef_client_t, Self>,
    render_handler: RenderHandler,
}

impl TerminalClient {
    pub fn new(render_handler: RenderHandler) -> Client {
        Client::new(Self {
            object: ptr::null_mut(),
            render_handler,
        })
    }
}

impl Clone for TerminalClient {
    fn clone(&self) -> Self {
        let object = unsafe {
            let rc_impl = &mut *self.object;
            rc_impl.interface.add_ref();
            self.object
        };
        Self {
            object,
            render_handler: self.render_handler.clone(),
        }
    }
}

impl cef::rc::Rc for TerminalClient {
    fn as_base(&self) -> &cef_dll_sys::cef_base_ref_counted_t {
        unsafe {
            let rc_impl = &*self.object;
            std::mem::transmute(&rc_impl.cef_object)
        }
    }
}

impl cef::WrapClient for TerminalClient {
    fn wrap_rc(&mut self, object: *mut RcImpl<cef_dll_sys::cef_client_t, Self>) {
        self.object = object;
    }
}

impl ImplClient for TerminalClient {
    fn get_raw(&self) -> *mut cef_dll_sys::_cef_client_t {
        self.object as *mut _
    }

    fn get_render_handler(&self) -> Option<cef::RenderHandler> {
        Some(self.render_handler.clone())
    }
}
