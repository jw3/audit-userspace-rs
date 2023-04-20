use crate::auparse::types::Type;
use crate::auparse::util::auparse_get_int;
use auparse_sys::{auparse_get_time, auparse_get_type, auparse_next_event, auparse_state_t};
use std::time::{Duration, SystemTime};

#[derive(Debug)]
pub struct Entry {
    pub etype: Type,
    pub time: SystemTime,
    pub pid: i32,
    pub uid: i32,
    pub gid: i32,
}

impl Entry {
    pub(crate) unsafe fn next(ptr: *mut auparse_state_t) -> Option<Entry> {
        match auparse_next_event(ptr) {
            1 => Some(Entry::parse(ptr)),
            _ => None,
        }
    }
    pub(crate) unsafe fn parse(ptr: *mut auparse_state_t) -> Entry {
        let tid = auparse_get_type(ptr) as u32;
        let ts = auparse_get_time(ptr) as u64;
        let pid = auparse_get_int(ptr, "pid");
        let uid = auparse_get_int(ptr, "uid");
        let gid = auparse_get_int(ptr, "gid");
        let time = std::time::UNIX_EPOCH + Duration::from_secs(ts as u64);
        Entry {
            etype: tid.into(),
            time,
            pid,
            uid,
            gid,
        }
    }
}
