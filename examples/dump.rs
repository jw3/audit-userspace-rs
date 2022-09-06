use audit_userspace_rs::auparse::error::Error;
use audit_userspace_rs::auparse::Log;

fn main() -> Result<(), Error> {
    let log = Log::new()?;
    log.for_each(|e| println!("{:?}", e));
    Ok(())
}
