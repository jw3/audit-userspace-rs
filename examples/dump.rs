use audit_userspace_rs::auparse::error::Error;
use audit_userspace_rs::auparse::Logs;

fn main() -> Result<(), Error> {
    let log = Logs::new()?;
    log.for_each(|e| println!("{:?}", e));
    Ok(())
}
