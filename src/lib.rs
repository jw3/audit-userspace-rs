use auparse_sys::*;
use std::ptr;
use std::time::{Duration, SystemTime};

pub struct Log {
    au: *mut auparse_state_t,
}

pub struct Entry {
    pub t: SystemTime,
}

impl Iterator for Log {
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            while auparse_next_event(self.au) > 0 {
                match auparse_get_type(self.au) as u32 {
                    AUDIT_SYSTEM_BOOT => {
                        let t = auparse_get_time(self.au);
                        let t = std::time::UNIX_EPOCH + Duration::from_secs(t as u64);
                        return Some(Self::Item { t });
                    }
                    _ => {}
                }
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
    pub fn new() -> Self {
        let au = unsafe { auparse_init(ausource_t_AUSOURCE_LOGS, ptr::null()) };
        Self { au }
    }
}
