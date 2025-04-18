use chrono::{Utc, TimeZone};

pub fn timestamp_to_elapsed(timestamp: i64) -> String {
    let now = Utc::now();
    let then = Utc.timestamp_opt(timestamp, 0).unwrap();
    let duration = now.signed_duration_since(then);

    if duration.num_seconds() < 60 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{} minute{} ago", duration.num_minutes(), if duration.num_minutes() == 1 { "" } else { "s" })
    } else if duration.num_hours() < 24 {
        format!("{} hour{} ago", duration.num_hours(), if duration.num_hours() == 1 { "" } else { "s" })
    } else if duration.num_days() < 30 {
        format!("{} day{} ago", duration.num_days(), if duration.num_days() == 1 { "" } else { "s" })
    } else if duration.num_days() < 365 {
        let months = duration.num_days() / 30;
        format!("{} month{} ago", months, if months == 1 { "" } else { "s" })
    } else {
        let years = duration.num_days() / 365;
        format!("{} year{} ago", years, if years == 1 { "" } else { "s" })
    }
}
