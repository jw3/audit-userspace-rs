use audit_userspace_rs::auparse::error::Error;
use audit_userspace_rs::auparse::error::Error::GeneralFail;
use audit_userspace_rs::auparse::rtype::Type::SystemBoot;
use audit_userspace_rs::auparse::Logs;
use chrono::{DateTime, Local};
use std::time::SystemTime;

/// Example that behaves like the ubiquitous uptime command
fn main() -> Result<(), Error> {
    let log = Logs::new()?;

    // filter the log to boots and take the last one
    let then = log
        .filter(|e| e.etype == SystemBoot)
        .last()
        .expect("[fail] no boot entries");

    // uptime from then till now
    let now = SystemTime::now();
    let uptime = now.duration_since(then.time)?;

    let datetime: DateTime<Local> = now.into();
    let duration = chrono::Duration::from_std(uptime)
        .map_err(|_| GeneralFail("Duration from uptime".to_string()))?;
    let hours = duration - chrono::Duration::days(duration.num_days());
    let min = duration - chrono::Duration::hours(duration.num_hours());

    // format stdout to look like `uptime`
    println!(
        " {} up {} days, {: >2}:{:02},",
        datetime.format("%T"),
        duration.num_days(),
        hours.num_hours(),
        min.num_minutes()
    );

    Ok(())
}
