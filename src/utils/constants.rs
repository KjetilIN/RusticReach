use colored::Colorize;
use crossterm::style::Stylize;
use once_cell::sync::Lazy;

pub const COMMAND_LINE_SYMBOL: &str = "$";
pub const MESSAGE_COMMAND_SYMBOL: &str = "/";
pub static ERROR_LOG: Lazy<String> = Lazy::new(|| Colorize::red("[ERROR]").to_string());
pub static INFO_LOG: Lazy<String> = Lazy::new(|| Colorize::green("[INFO]").to_string());
pub static WARNING_LOG: Lazy<String> = Lazy::new(|| Colorize::yellow("[WARNING]").to_string());
pub static SELF_USER: Lazy<String> = Lazy::new(|| Colorize::yellow("<YOU>").to_string());
pub static SERVER_INFO: Lazy<String> =
    Lazy::new(|| Colorize::bold("[SERVER]").yellow().to_string());

pub static MESSAGE_LINE_SYMBOL: Lazy<String> = Lazy::new(|| Colorize::blue(">").to_string());
pub const DEFAULT_SERVER_PORT: &str = "8080";

pub fn server_message(content: &str) -> String {
    return format!("{} {}", *SERVER_INFO, content).yellow().to_string();
}
