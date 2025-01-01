use chrono::{DateTime, Local};
use colored::Colorize;
use std::time::SystemTime;

use crate::utils::constants::SELF_USER;

pub fn format_message_string(
    user_name: &str,
    user_name_color: (u8, u8, u8),
    message: &str,
) -> String {
    let now = SystemTime::now();

    // Convert SystemTime to a DateTime with the local timezone
    let datetime: DateTime<Local> = now.into();

    // Format the timestamp, as light gray color
    let time_stamp = format!(
        "{}:{}:{}",
        datetime.format("%H"),
        datetime.format("%M"),
        datetime.format("%S")
    )
    .truecolor(211, 211, 211)
    .to_string();

    // Create user name string, by adding brackets and setting the color
    //TODO: random light color generator for username text
    let user = format!("<{}>", user_name)
        .truecolor(user_name_color.0, user_name_color.1, user_name_color.2)
        .to_string();

    // Print message from self in the terminal
    println!("{} {} {}", &time_stamp, *SELF_USER, message);

    // Return the formatted string
    return format!("{} {} {}", &time_stamp, &user, message);
}
