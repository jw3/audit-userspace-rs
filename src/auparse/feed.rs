use std::ffi::{c_void, CStr};
use std::ptr::NonNull;
use std::{ptr, thread};

use crossbeam_channel::internal::SelectHandle;
use crossbeam_channel::{select, Receiver, Sender};

use auparse_sys::*;

use crate::auparse::entry::Entry;
use crate::auparse::error::Error;

pub type Done = ();

#[derive(Clone)]
pub struct Stream {
    pub tx: Sender<String>,
    pub rx: Receiver<Entry>,
    e_tx: Sender<Entry>,
}

pub struct Feed {
    au: NonNull<auparse_state_t>,
}

unsafe impl Send for Feed {}

impl Drop for Feed {
    fn drop(&mut self) {
        println!("==============--------------- DROPPING --------------================");
        unsafe {
            auparse_destroy(self.au.as_ptr());
        }
    }
}

pub type StreamRef = (Sender<Done>, Box<Stream>);

impl Feed {
    pub fn new() -> Self {
        let au = unsafe { auparse_init(ausource_t_AUSOURCE_FEED, ptr::null()) };
        Self {
            au: NonNull::new(au).expect("non null au"),
        }
    }

    // consume the Feed
    // set up the au with a callback
    // start a thread that monitors the source, passing the input to the au
    // the callback receives log entries and passes them to the sink
    // the thread is closed and the au destroyed when the source closes
    // when closing flushing is attempted if the sink is still valid
    pub fn stream(self) -> Result<StreamRef, Error> {
        // source
        let (source_tx, source_rx) = crossbeam_channel::unbounded();
        // sink
        let (sink_tx, sink_rx) = crossbeam_channel::unbounded();
        // done
        let (done, shutdown) = crossbeam_channel::bounded(0);

        let stream = Box::new(Stream {
            tx: source_tx,
            rx: sink_rx,
            e_tx: sink_tx,
        });

        unsafe {
            let user_data = Box::into_raw(stream.clone()) as *mut c_void;
            auparse_add_callback(self.au.as_ptr(), Some(callback), user_data, Some(cleanup));
        }

        thread::spawn(move || {
            let feed = self;
            let mut count = 0;
            loop {
                select! {
                    recv(shutdown) -> _ => {
                        println!("((((((((((((( SHUTTING DOWN ))))))))))))))))");
                        break
                    },
                    recv(source_rx) -> e => if let Ok(txt)= e {
                        count += 1;
                        unsafe { auparse_feed(feed.au.as_ptr(), txt.as_ptr() as *const i8, txt.len().try_into().unwrap()); }
                    },
                }
            }
            println!("handled {count} events");
            unsafe {
                //auparse_feed_age_events(feed.au.as_ptr());
            }
            println!("should drop the Feed");
        });

        Ok((done, stream))
    }
}

// the callback will
// todo;; allow callback logic to be injected to the Feed prior to stream() being called
unsafe extern "C" fn callback(
    au: *mut auparse_state_t,
    cb_event_type: auparse_cb_event_t,
    user_data: *mut ::std::os::raw::c_void,
) {
    if !user_data.is_null() && cb_event_type == auparse_cb_event_t_AUPARSE_CB_EVENT_READY {
        // let x = audit_msg_type_to_name(auparse_get_type(au));
        // let name = CStr::from_ptr(x).to_str().unwrap();
        // println!("{name}");

        let vp = user_data as *mut Stream;
        if let Some(e) = Entry::next(au) {
            (*vp).e_tx.send(e);
        }
    }
}

extern "C" fn cleanup(_user_data: *mut ::std::os::raw::c_void) {
    // todo;; send a Done event to the stream?
    println!("========== callback cleanup ==========");
}
