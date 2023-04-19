use crate::auparse::entry::Entry;
use crate::auparse::error::Error;
use crate::auparse::error::Error::NativeInitFail;
use auparse_sys::*;
use std::ptr;
use std::ptr::NonNull;

pub struct Logs {
    au: NonNull<auparse_state_t>,
}

impl Iterator for Logs {
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe { Entry::next(self.au.as_ptr()) }
    }
}

impl Drop for Logs {
    fn drop(&mut self) {
        unsafe {
            auparse_destroy(self.au.as_ptr());
        }
    }
}

impl Logs {
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
