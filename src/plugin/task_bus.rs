// src/plugin/task_bus.rs
// luo9 SDK FFI 绑定与安全封装

use libc::c_char;
use std::ffi::{CStr, CString};

unsafe extern "C" {
    unsafe fn luo9_bus_init(cap: usize) -> libc::c_int;
    unsafe fn luo9_bus_publish(topic: *const c_char, payload: *const c_char) -> libc::c_int;
    unsafe fn luo9_bus_pop(topic: *const c_char) -> *mut c_char;
    unsafe fn luo9_bus_free_string(ptr: *mut c_char);
}

pub struct Bus;

impl Bus {
    pub fn init(cap: usize) -> Result<(), BusError> {
        let ret = unsafe { luo9_bus_init(cap) };
        match ret {
            0 => Ok(()),
            _ => Err(BusError::InitFailed),
        }
    }

    pub fn topic<'a>(name: &'a str) -> Topic<'a> {
        Topic { name }
    }
}

pub struct Topic<'a> {
    name: &'a str,
}

impl<'a> Topic<'a> {
    pub fn publish(&self, payload: &str) -> Result<(), BusError> {
        let topic = CString::new(self.name).map_err(|_| BusError::InvalidString)?;
        let payload = CString::new(payload).map_err(|_| BusError::InvalidString)?;

        let ret = unsafe { luo9_bus_publish(topic.as_ptr(), payload.as_ptr()) };
        match ret {
            0 => Ok(()),
            -2 => Err(BusError::NotInitialized),
            _ => Err(BusError::PublishFailed),
        }
    }

    pub fn pop(&self) -> Option<String> {
        let topic = CString::new(self.name).ok()?;
        let ptr = unsafe { luo9_bus_pop(topic.as_ptr()) };
        if ptr.is_null() {
            return None;
        }
        let msg = unsafe {
            let s = CStr::from_ptr(ptr).to_string_lossy().to_string();
            luo9_bus_free_string(ptr);
            s
        };
        Some(msg)
    }

    pub fn publish_fmt(&self, payload: impl ToString) -> Result<(), BusError> {
        self.publish(&payload.to_string())
    }
}

#[derive(Debug)]
pub enum BusError {
    InitFailed,
    PublishFailed,
    NotInitialized,
    InvalidString,
}
