use auparse_sys::*;
use std::cell::Cell;
use std::error::Error;
use std::ffi::c_void;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, SystemTime};
use std::{ptr, thread};

struct UserData {
    tx: Rc<Sender<Entry>>,
}

pub struct Log {
    au: *mut auparse_state_t,
    user_data: Cell<Option<UserData>>,
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
    pub fn parse() -> Self {
        let au = unsafe { auparse_init(ausource_t_AUSOURCE_LOGS, ptr::null()) };
        Self {
            au,
            user_data: Cell::new(None),
        }
    }

    pub fn feed() -> Self {
        let au = unsafe { auparse_init(ausource_t_AUSOURCE_FEED, ptr::null()) };
        Self {
            au,
            user_data: Cell::new(None),
        }
    }
}

impl Log {
    pub fn rx(&self) -> Result<Receiver<Entry>, Box<dyn Error>> {
        let (tx, rx) = mpsc::channel();
        unsafe {
            let rc = Rc::new(tx);
            self.user_data.set(Some(UserData { tx: rc.clone() }));
            let user_data = Rc::into_raw(rc) as *mut c_void;
            auparse_add_callback(self.au, Some(callback), user_data, Some(cleanup));
            auparse_feed_age_events(self.au);

            thread::spawn(|| loop {
                //     libc::timeval {
                //         tv_sec: duration.as_secs() as i64,
                //         tv_usec: duration.subsec_micros() as i32,
                //     }
                //
                //     let mut raw_fd_set = mem::MaybeUninit::<libc::fd_set>::uninit();
                //     libc::FD_ZERO(raw_fd_set.as_mut_ptr());
                //
                //     libc::select(
                //         nfds,
                //         to_fdset_ptr(readfds),
                //         to_fdset_ptr(writefds),
                //         to_fdset_ptr(errorfds),
                //         to_ptr::<libc::timeval>(timeout) as *mut libc::timeval,
                //     )
                // } {
                //           b        -1 => Err(io::Error::last_os_error()),
                //                   res => Ok(res as usize),
                //               }

                println!("** enter **");
                let buf = [0; MAX_AUDIT_MESSAGE_LENGTH as usize];
                if libc::read(
                    0,
                    buf.as_slice().as_ptr() as *mut c_void,
                    MAX_AUDIT_MESSAGE_LENGTH.try_into().unwrap(),
                ) > 0
                {
                    println!("=====");
                }

                println!("** exit **");
            });
        }
        Ok(rx)
    }
}

unsafe extern "C" fn callback(
    au: *mut auparse_state_t,
    cb_event_type: auparse_cb_event_t,
    user_data: *mut ::std::os::raw::c_void,
) {
    if !user_data.is_null() {
        let vp = &*(user_data as *mut UserData);
        vp.tx.send(Entry {
            t: SystemTime::now(),
        });
    }

    println!("========== callback ==========");
}

extern "C" fn cleanup(user_data: *mut ::std::os::raw::c_void) {
    println!("========== callback cleanup ==========");
}
