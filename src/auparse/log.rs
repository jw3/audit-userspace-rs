use crate::auparse::entry::Entry;
use crate::auparse::error::Error;
use crate::auparse::error::Error::NativeInitFail;
use auparse_sys::*;
use std::ptr;
use std::ptr::NonNull;

pub struct Log {
    au: NonNull<auparse_state_t>,
}

impl Iterator for Log {
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe { Entry::next(self.au.as_ptr()) }
    }
}

impl Drop for Log {
    fn drop(&mut self) {
        unsafe {
            auparse_destroy(self.au.as_ptr());
        }
    }
}

impl Log {
    pub fn new() -> Result<Self, Error> {
        let au = unsafe { auparse_init(ausource_t_AUSOURCE_LOGS, ptr::null()) };
        if au.is_null() {
            Err(NativeInitFail)
        } else {
            Ok(Self {
                au: NonNull::new(au).expect("non null au"),
            })
        }
    }
}
