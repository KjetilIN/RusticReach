use actix_web::web::Bytes;
use awc::ws::{self};
use colored::Colorize;
use futures_util::{SinkExt as _, StreamExt as _};
use once_cell::sync::Lazy;
use rustic_reach::{
    client::config::{parse_client_config, ClientConfig},
    shared::formatted_messages::format_message_string,
};
use std::{
    env,
    io::{self, Write},
    process::exit,
    sync::Arc,
    thread::{self, sleep},
    time::Duration,
};
use tokio::{select, sync::mpsc, task::LocalSet};
use tokio_stream::wrappers::UnboundedReceiverStream;

const COMMAND_LINE_SYMBOL: &str = "$";
const MESSAGE_COMMAND_SYMBOL: &str = "/";
static ERROR_LOG: Lazy<String> = Lazy::new(|| "[ERROR]".red().to_string());
static INFO_LOG: Lazy<String> = Lazy::new(|| "[INFO]".green().to_string());
static WARNING_LOG: Lazy<String> = Lazy::new(|| "[WARNING]".yellow().to_string());
static MESSAGE_LINE_SYMBOL: Lazy<String> = Lazy::new(|| ">".blue().to_string());
const DEFAULT_SERVER_PORT: &str = "8080";

fn handle_message_commands(
    input: String,
    client_config: &ClientConfig,
    room_name: &mut Option<String>,
) {
    // Message command only if the command starts with the command symbol
    // This allows users to execute commands when they are messaging
    if input.starts_with(MESSAGE_COMMAND_SYMBOL) {
        // Handle given command
        match input.as_str() {
            "/disconnect" => {
                println!("{} Disconnecting...", *INFO_LOG);
            }
            _ => {
                println!("{} Unknown command", *ERROR_LOG);
            }
        }
    } else {
        // The input is text and should be sent to the server
        let user_name = client_config.get_user_name(room_name);
        let message = format_message_string(user_name, (250, 0, 0), &input);
    }
}

/**
 * Validates the input arguments of the
 *
 * Logs error and info
 *
 * Returns either the parsed client config or an error.
 */
pub fn validate_args() -> Result<ClientConfig, ()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 || &args[1] != "-c" || args[2].is_empty() {
        println!("{} Usage: <program> -c <client.yml>", *ERROR_LOG);
        return Err(());
    }

    let file_path = &args[2];
    if let Some(config) = parse_client_config(&file_path) {
        return Ok(config);
    } else {
        println!(
            "{} Provided client config {} could not be parsed",
            *ERROR_LOG, file_path
        );
        return Err(());
    }
}

#[tokio::main]
async fn main() {
    let client_config: ClientConfig = validate_args().unwrap_or_else(|_| {
        exit(1);
    });

    println!("{} Client config parsed", *INFO_LOG);

    // If a default server and auto connect is set to true, then do connection
    if let Some(server_options) = client_config.get_default_server() {
        if server_options.should_auto_connect() {
            println!("{} Auto connecting to default server...", *INFO_LOG);
            //connect(server_options.ip(), DEFAULT_SERVER_PORT.to_string()).await;
        }
        println!(
            "{} Default server detected. Use /connect (without args) to connect to the server.",
            *INFO_LOG
        );
    }

    // Infinite input loop
    loop {
        // Print the command line symbol
        print!("{} ", COMMAND_LINE_SYMBOL);

        // Flush the standard output to make sure the symbol appears immediately
        std::io::stdout().flush().expect("Failed to flush stdout");

        let mut input = String::new();

        // Read line or fail
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        // Trim input
        let trimmed_input = input.trim();

        // If the current state is connected, we handle input as messages to the server
        // Otherwise we handle inputs as commands
        // handle_client_command(trimmed_input).await;
    }
}
