use audit_userspace_rs::auparse::rtype::Type::SystemBoot;
use audit_userspace_rs::auparse::Log;
use chrono::{DateTime, Local};
use std::error::Error;
use std::time::SystemTime;

fn main() -> Result<(), Box<dyn Error>> {
    let log = Log::new();
    let boot = log
        .filter(|e| e.etype == SystemBoot)
        .last()
        .expect("no boot entries");

    let now = SystemTime::now();
    let uptime = now.duration_since(boot.time)?;

    let datetime: DateTime<Local> = now.into();
    let duration = chrono::Duration::from_std(uptime)?;
    let hours = duration - chrono::Duration::days(duration.num_days());
    let min = duration - chrono::Duration::hours(duration.num_hours());

    println!(
        " {} up {} days, {: >2}:{:02},",
        datetime.format("%T"),
        duration.num_days(),
        hours.num_hours(),
        min.num_minutes()
    );

    Ok(())
}
