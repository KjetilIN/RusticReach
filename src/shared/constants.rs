use colored::Colorize;
use once_cell::sync::Lazy;

pub const COMMAND_LINE_SYMBOL: &str = "$";
pub const MESSAGE_COMMAND_SYMBOL: &str = "/";
pub static ERROR_LOG: Lazy<String> = Lazy::new(|| "[ERROR]".red().to_string());
pub static INFO_LOG: Lazy<String> = Lazy::new(|| "[INFO]".green().to_string());
pub static WARNING_LOG: Lazy<String> = Lazy::new(|| "[WARNING]".yellow().to_string());
pub static SELF_USER: Lazy<String> = Lazy::new(|| "<YOU>".yellow().to_string());

pub static MESSAGE_LINE_SYMBOL: Lazy<String> = Lazy::new(|| ">".blue().to_string());
pub const DEFAULT_SERVER_PORT: &str = "8080";
