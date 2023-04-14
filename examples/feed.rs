use std::env::args;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::thread;
use std::time::Duration;

use clap::Parser;
use crossbeam_channel::internal::SelectHandle;

use audit_userspace_rs::auparse::Feed;

#[derive(Parser)]
#[clap(name = "Audit Feed")]
struct Opts {
    pub log: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();

    let (_done, stream) = Feed::new().stream()?;
    let rx = stream.rx.clone();
    thread::spawn(move || loop {
        for e in &rx {
            println!("-- {e:?}");
        }
    });

    let f = File::open(opts.log).expect("failed to open file");
    let buff = BufReader::new(f);

    for line in buff.lines() {
        if let Some(e) = stream.tx.send(format!("{}\n", line?)).err() {
            println!("error: {e} {}", stream.tx.is_ready());
        }
    }
    Ok(())
}
