use rustic_reach::{
    client::{
        config::ClientConfig,
        runtime::connect,
    },
    utils::{args::validate_args, constants::{
        COMMAND_LINE_SYMBOL, DEFAULT_SERVER_PORT, INFO_LOG, WARNING_LOG,
    }},
};
use std::{
    io::{self, Write},
    process::exit,
};

#[tokio::main]
async fn main() {
    let client_config: ClientConfig = validate_args().unwrap_or_else(|message| {
        println!("{}", message);
        exit(1);
    });

    println!("{} Client config parsed", *INFO_LOG);

    // Check if we are asked to validate the server checksum
    if client_config.get_validate_server() {
        // TODO: implement checksum validation of server repo
        println!("{} Validating server repo not implemented!", *WARNING_LOG);
    }

    // If a default server and auto connect is set to true, then do connection
    if let Some(server_options) = client_config.get_default_server() {
        if server_options.should_auto_connect() {
            println!("{} Auto connecting to default server...", *INFO_LOG);
            connect(
                server_options.ip(),
                DEFAULT_SERVER_PORT.to_string(),
                client_config.into(),
            )
            .await;
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
        //let trimmed_input = input.trim();
        // If the current state is connected, we handle input as messages to the server
        // Otherwise we handle inputs as commands
        // handle_client_command(trimmed_input).await;
    }
}
