use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::thread;

use clap::Parser;
use crossbeam_channel::internal::SelectHandle;

use audit_userspace_rs::auparse::feed::{Input, Output};
use audit_userspace_rs::auparse::Feed;

#[derive(Parser)]
#[clap(name = "Audit Feed")]
struct Opts {
    pub log: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();

    let stream = Feed::new().stream()?;
    let rx = stream.rx.clone();

    // entry rx thread displays parsed entries
    let h = thread::spawn(move || {
        let mut count = 0;
        for e in &rx {
            match e {
                Output::Done => break,
                Output::Parsed(e) => {
                    count += 1;
                    println!("{e:?}")
                }
            }
        }
        println!("output: {count}");
    });

    // read entries from a file, push them into the stream
    let f = File::open(opts.log).expect("failed to open file");
    let buff = BufReader::new(f);
    for line in buff.lines() {
        if let Some(e) = stream.tx.send(Input::Raw(format!("{}\n", line?))).err() {
            println!("error: {e} {}", stream.tx.is_ready());
        }
    }

    // signal the input completed
    stream.tx.send(Input::Done)?;

    // wait for the rx thread to close
    h.join().expect("join rx thread");

    Ok(())
}
