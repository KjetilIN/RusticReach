use rustic_reach::{
    client::{
        client_runtime::connect,
        config::{parse_client_config, ClientConfig},
    },
    utils::constants::{
        COMMAND_LINE_SYMBOL, DEFAULT_SERVER_PORT, ERROR_LOG, INFO_LOG, WARNING_LOG,
    },
};
use std::{
    env,
    io::{self, Write},
    process::exit,
};

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
