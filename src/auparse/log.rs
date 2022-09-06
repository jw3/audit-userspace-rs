use crate::auparse::error::Error;
use crate::auparse::error::Error::NativeInitFail;
use crate::auparse::rtype::Type;
use auparse_sys::*;
use std::ffi::CString;
use std::ptr;
use std::time::{Duration, SystemTime};

pub struct Log {
    au: *mut auparse_state_t,
}

#[derive(Debug)]
pub struct Entry {
    pub etype: Type,
    pub time: SystemTime,
    pub pid: i32,
    pub uid: i32,
    pub gid: i32,
}

impl Iterator for Log {
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            while auparse_next_event(self.au) > 0 {
                let tid = auparse_get_type(self.au) as u32;
                let ts = auparse_get_time(self.au) as u64;
                let pid = auparse_get_int(self.au, "pid");
                let uid = auparse_get_int(self.au, "uid");
                let gid = auparse_get_int(self.au, "gid");
                let time = std::time::UNIX_EPOCH + Duration::from_secs(ts as u64);
                return Some(Self::Item {
                    etype: tid.into(),
                    time,
                    pid,
                    uid,
                    gid,
                });
            }
        }
        None
    }
}

impl Drop for Log {
    fn drop(&mut self) {
        unsafe {
            auparse_destroy(self.au);
        }
    }
}

impl Log {
    pub fn new() -> Result<Self, Error> {
        let au = unsafe { auparse_init(ausource_t_AUSOURCE_LOGS, ptr::null()) };
        if au.is_null() {
            Err(NativeInitFail)
        } else {
            Ok(Self { au })
        }
    }
}

unsafe fn auparse_get_int(au: *mut auparse_state_t, field: &str) -> i32 {
    let str = CString::new(field).expect("CString");
    let tpid = auparse_find_field(au, str.as_ptr());
    if !tpid.is_null() {
        auparse_get_field_int(au) as i32
    } else {
        -1
    }
}
