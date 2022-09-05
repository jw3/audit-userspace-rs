use audit_userspace_rs::Log;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let log = Log::feed();
    for e in log.rx()? {
        println!("e: {:?}", e.t);
    }

    Ok(())
}
