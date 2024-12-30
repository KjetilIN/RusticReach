use actix_web::web::Bytes;
use awc::ws::{self};
use colored::Colorize;
use futures_util::{SinkExt as _, StreamExt as _};
use once_cell::sync::Lazy;
use std::{
    io::{self, Write},
    process::exit,
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

async fn connect(server_ip: String, server_port: String) {
    let local = LocalSet::new();

    local.spawn_local(async move {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        let mut cmd_rx = UnboundedReceiverStream::new(cmd_rx);

        // run blocking terminal input reader on a separate thread
        let input_thread = thread::spawn(move || loop {
            let mut cmd = String::with_capacity(32);

            // Print message line symbol
            print!("{} ", *MESSAGE_LINE_SYMBOL);

            // Flush the output
            std::io::stdout().flush().expect("Failed to flush stdout");

            // Read input
            if io::stdin().read_line(&mut cmd).is_err() {
                println!("{} could not read message input", *ERROR_LOG);
                return;
            }

            // Handle input
            handle_message_commands(cmd.clone());

            // Send ...
            cmd_tx.send(cmd).unwrap();
        });

        // Format the websocket url
        let ws_url = format!("ws://{server_ip}:{server_port}/ws");

        println!("{} Trying to connect to {} ...", *INFO_LOG, ws_url);

        // Connect to the server
        let (res, mut ws) = awc::Client::new().ws(ws_url).connect().await.unwrap();

        println!("{} response: {res:?}", *INFO_LOG);

        // Sleep for a bit before clearing the terminal
        sleep(Duration::from_millis(300));

        // Clear screen
        println!("{}[2J", 27 as char);

        // Handle incoming messages
        loop {
            select! {
                Some(msg) = ws.next() => {
                    match msg {
                        Ok(ws::Frame::Text(txt)) => {
                            match String::from_utf8(txt.to_vec()) {
                                Ok(valid_str) => {
                                    println!("{}", valid_str);
                                }
                                Err(err) => {
                                    println!("{} Failed to parse text frame: {}", *ERROR_LOG, err);
                                }
                            }
                        }
                        Ok(ws::Frame::Ping(_)) => {
                            ws.send(ws::Message::Pong(Bytes::new())).await.unwrap();
                        }
                        _ => {}
                    }
                }
                Some(cmd) = cmd_rx.next() => {
                    if cmd.is_empty() {
                        continue;
                    }

                    ws.send(ws::Message::Text(cmd.into())).await.unwrap();
                }
                else => break,
            }
        }

        input_thread.join().unwrap();
    });

    local.await; // Wait for the LocalSet to complete
}

async fn handle_client_command(command: &str) {
    let command_parts: Vec<&str> = command.split_ascii_whitespace().collect();

    match command_parts.as_slice() {
        ["exit"] => {
            println!("{} Exiting the command", *INFO_LOG);
            exit(0);
        }
        ["connect", ..] => {
            if command_parts.len() != 2 {
                println!("{} Please provide server IP", *ERROR_LOG);
                return;
            }

            // Connect to server command
            let ip = command_parts[1];
            println!("{} Connecting to server at IP {}", *INFO_LOG, ip);

            // Do connection
            connect(ip.to_string(), DEFAULT_SERVER_PORT.to_string()).await;
        }
        _ => {
            println!("{} Unknown command", *ERROR_LOG);
        }
    }
}

fn handle_message_commands(input: String) {
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
        // TODO: send message to server
    }
}

#[tokio::main]
async fn main() {
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
        handle_client_command(trimmed_input).await;
    }
}
