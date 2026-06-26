use std::{ffi::c_void, os::raw};

#[derive(Debug, Clone, Copy)]
pub struct OpenGLPlatform {
    pub display: *mut c_void,
    pub context: *mut c_void,
    pub surface: *mut c_void,
    pub get_proc_address: unsafe extern "C" fn(name: *const raw::c_char) -> *const c_void,
    pub get_current_context: unsafe extern "C" fn() -> *const c_void,
}
