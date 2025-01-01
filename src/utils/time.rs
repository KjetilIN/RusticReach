use chrono::{DateTime, Local};
use std::time::SystemTime;

pub fn get_time_string() -> String {
    let now = SystemTime::now();

    // Convert SystemTime to a DateTime with the local timezone
    let datetime: DateTime<Local> = now.into();

    // Format the timestamp, as light gray color
    return format!(
        "{}:{}:{}",
        datetime.format("%H"),
        datetime.format("%M"),
        datetime.format("%S")
    );
}
