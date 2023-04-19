use std::ffi::c_void;
use std::ptr::NonNull;
use std::{ptr, thread};

use crossbeam_channel::{Receiver, Sender};

use auparse_sys::*;

use crate::auparse::entry::Entry;
use crate::auparse::error::Error;

#[derive(Clone)]
pub struct Stream {
    pub tx: Sender<Input>,
    pub rx: Receiver<Output>,
    count: u32,
    e_tx: Sender<Output>,
}

pub enum Input {
    Raw(String),
    Done,
}

pub enum Output {
    Parsed(Entry),
    Done,
}

pub struct Feed {
    au: NonNull<auparse_state_t>,
}

unsafe impl Send for Feed {}

impl Drop for Feed {
    fn drop(&mut self) {
        unsafe {
            auparse_destroy(self.au.as_ptr());
        }
    }
}

pub type StreamRef = Box<Stream>;

impl Feed {
    pub fn new() -> Self {
        let au = unsafe { auparse_init(ausource_t_AUSOURCE_FEED, ptr::null()) };
        Self {
            au: NonNull::new(au).expect("non null au"),
        }
    }

    pub fn stream(self) -> Result<StreamRef, Error> {
        let (source_tx, source_rx) = crossbeam_channel::unbounded();
        let (sink_tx, sink_rx) = crossbeam_channel::unbounded();

        let stream = Box::new(Stream {
            tx: source_tx,
            rx: sink_rx,
            count: 0,
            e_tx: sink_tx,
        });

        unsafe {
            let user_data = Box::into_raw(stream.clone()) as *mut c_void;
            auparse_add_callback(self.au.as_ptr(), Some(callback), user_data, Some(cleanup));
        }

        thread::spawn(move || {
            let feed = self;
            let mut count = 0;
            'outer: loop {
                for e in &source_rx {
                    match e {
                        Input::Raw(txt) => {
                            count += 1;
                            unsafe {
                                auparse_feed(
                                    feed.au.as_ptr(),
                                    txt.as_ptr() as *const i8,
                                    txt.len().try_into().unwrap(),
                                );
                            }
                        }
                        Input::Done => {
                            break 'outer;
                        }
                    }
                }
            }
            println!("received: {count}");
            unsafe {
                auparse_feed_age_events(feed.au.as_ptr());
                auparse_flush_feed(feed.au.as_ptr());
            }
            println!("stream closed");
        });

        Ok(stream)
    }
}

// todo;; allow callback logic to be injected to the Feed prior to stream() being called
unsafe extern "C" fn callback(
    au: *mut auparse_state_t,
    cb_event_type: auparse_cb_event_t,
    user_data: *mut c_void,
) {
    if !user_data.is_null() && cb_event_type == auparse_cb_event_t_AUPARSE_CB_EVENT_READY {
        // let x = audit_msg_type_to_name(auparse_get_type(au));
        // let name = CStr::from_ptr(x).to_str().unwrap();
        // println!("{name}");
        let stream = user_data as *mut Stream;
        if let Some(e) = Entry::next(au) {
            (*stream).count += 1;
            (*stream).e_tx.send(Output::Parsed(e));
        }
    }
}

extern "C" fn cleanup(user_data: *mut c_void) {
    unsafe {
        let stream = Box::from_raw(user_data as *mut Stream);
        stream.e_tx.send(Output::Done);
        println!("stream.count: {}", stream.count);
    }
}
