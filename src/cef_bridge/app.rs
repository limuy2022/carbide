use cef::rc::{Rc, RcImpl};

#[derive(Debug)]
pub struct TerminalApp {
    object: *mut RcImpl<cef_dll_sys::cef_app_t, Self>,
}

impl TerminalApp {
    pub fn new() -> Self {
        Self {
            object: std::ptr::null_mut(),
        }
    }
}

impl Clone for TerminalApp {
    fn clone(&self) -> Self {
        let object = unsafe {
            let rc_impl = &mut *self.object;
            rc_impl.interface.add_ref();
            self.object
        };
        Self { object }
    }
}

impl cef::rc::Rc for TerminalApp {
    fn as_base(&self) -> &cef_dll_sys::cef_base_ref_counted_t {
        unsafe {
            let rc_impl = &*self.object;
            std::mem::transmute(&rc_impl.cef_object)
        }
    }
}

impl cef::WrapApp for TerminalApp {
    fn wrap_rc(&mut self, object: *mut RcImpl<cef_dll_sys::cef_app_t, Self>) {
        self.object = object;
    }
}

impl cef::ImplApp for TerminalApp {
    fn get_raw(&self) -> *mut cef_dll_sys::_cef_app_t {
        self.object as *mut _
    }
}
