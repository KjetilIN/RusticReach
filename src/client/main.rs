use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use rustic_reach::{
    client::{config::ClientConfig, runtime::connect},
    utils::{
        args::validate_args,
        constants::{DEFAULT_SERVER_PORT, INFO_LOG, WARNING_LOG},
        terminal_ui::TerminalUI,
    },
};
use std::{
    process::exit,
    sync::{Arc, Mutex},
};

#[tokio::main]
async fn main() {
    let client_config: ClientConfig = validate_args().unwrap_or_else(|message| {
        println!("{}", message);
        exit(1);
    });

    // Initialize terminal UI
    enable_raw_mode().unwrap();

    // Create a terminal ui behind mutex
    let terminal_ui = Arc::new(Mutex::new(TerminalUI::new().unwrap()));
    if let Ok(mut ui) = terminal_ui.lock() {
        ui.add_message(format!("{} Client config parsed", *INFO_LOG));
    }

    // Check if we are asked to validate the server checksum
    if client_config.get_validate_server() {
        // TODO: implement checksum validation of server repo
        if let Ok(mut ui) = terminal_ui.lock() {
            ui.add_message(format!(
                "{} Validating server repo not implemented!",
                *WARNING_LOG
            ));
        }
    }

    // If a default server and auto connect is set to true, then do connection
    if let Some(server_options) = client_config.get_default_server() {
        if server_options.should_auto_connect() {
            println!("{} Auto connecting to default server...", *INFO_LOG);
            connect(
                server_options.ip(),
                DEFAULT_SERVER_PORT.to_string(),
                client_config.into(),
                terminal_ui,
            )
            .await;
        }
        println!(
            "{} Default server detected. Use /connect (without args) to connect to the server.",
            *INFO_LOG
        );
    }

    // End of program
    disable_raw_mode().unwrap();
}
