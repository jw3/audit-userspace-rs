use auparse_sys::*;
use chrono::{DateTime, Local};
use std::error::Error;
use std::ptr;
use std::time::{Duration, SystemTime};

fn main() -> Result<(), Box<dyn Error>> {
    unsafe {
        let au = auparse_init(ausource_t_AUSOURCE_LOGS, ptr::null());

        let mut t = 0;
        while auparse_next_event(au) > 0 {
            match auparse_get_type(au) as u32 {
                AUDIT_SYSTEM_BOOT => t = auparse_get_time(au),
                _ => {}
            }
        }
        auparse_destroy(au);

        let boot = std::time::UNIX_EPOCH + Duration::from_secs(t as u64);
        let now = SystemTime::now();
        let uptime = now.duration_since(boot)?;

        let datetime: DateTime<Local> = now.into();
        let duration = chrono::Duration::from_std(uptime)?;
        let hours = duration - chrono::Duration::days(duration.num_days());
        let min = duration - chrono::Duration::hours(duration.num_hours());

        println!(
            "{} up {} days, {}:{}",
            datetime.format("%T"),
            duration.num_days(),
            hours.num_hours(),
            min.num_minutes()
        );
    }

    Ok(())
}
