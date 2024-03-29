use log::{error, info, LevelFilter};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::Config;
use std::error::Error;
use std::io::{stdin, Read};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use mio::unix::SourceFd;
use mio::{Events, Interest, Poll, Token};
use signal_hook::consts::{SIGHUP, SIGINT};

use audit_userspace_rs::auparse::feed::{Input, Output};
use audit_userspace_rs::auparse::Feed;

/**
 * Example ported from fapolicyd contrib @
 * https://github.com/linux-audit/audit-userspace/blob/master/contrib/plugin/audisp-example.c
 *
 * This is a sample program to demonstrate several concepts of how to
 * write an audispd plugin using libauparse. It can be tested by using a
 * file of raw audit records. You can generate the test file like:
 *
 * ausearch --start today --raw > test.log.
 *
 * Then you can test this app by: cat test.log | ./audisp-example
 */
fn main() -> Result<(), Box<dyn Error>> {
    log4rs::init_config(setup_logging()?)?;

    let mut _config = load_config()?;

    let stream = Feed::new().stream()?;
    let rx = stream.rx.clone();

    const READ: Token = Token(0);

    let mut events = Events::with_capacity(128);
    let mut poll = Poll::new()?;
    poll.registry()
        .register(&mut SourceFd(&0), READ, Interest::READABLE)?;

    let stop = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(SIGINT, Arc::clone(&stop)).unwrap();

    let hup = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(SIGHUP, Arc::clone(&hup)).unwrap();

    // consume parsed entries, logging them to disk
    let consumer = thread::spawn(move || {
        for e in &rx {
            match e {
                Output::Done => break,
                Output::Parsed(e) => info!("{e:?}"),
            }
        }
    });

    // stdin reading buffer
    let mut buff = [0; 1024];

    // main input parsing loop
    loop {
        if let Err(e) = poll.poll(&mut events, Some(Duration::from_secs(1))) {
            if e.kind() == std::io::ErrorKind::Interrupted {
                continue;
            } else {
                panic!("Poll error: {}", e);
            }
        }

        if stop.load(Ordering::Relaxed) {
            stream.tx.send(Input::Done).expect("stream.tx.send Done");
            break;
        }

        if hup.load(Ordering::Relaxed) {
            _config = load_config()?;
        }

        for e in events.iter() {
            match e.token() {
                READ => {
                    while stdin().read(&mut buff)? > 0 {
                        let string = String::from_utf8(buff.to_vec()).unwrap();
                        if let Some(e) = stream.tx.send(Input::Raw(format!("{string}\n"))).err() {
                            error!("stream.tx.send: {e}");
                        }
                    }
                }
                _ => println!("unknown token: {:?}", e),
            }
        }
    }
    stream.tx.send(Input::Done).expect("send Input::Done");
    consumer.join().expect("joining consumer");

    Ok(())
}

fn load_config() -> Result<(), Box<dyn Error>> {
    // todo;; load config
    Ok(())
}

fn setup_logging() -> Result<Config, Box<dyn Error>> {
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build("output.log")?;

    Ok(Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))?)
}
