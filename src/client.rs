use std::{io::{self, Write}, process::{exit, Command}};

const COMMAND_LINE_SYMBOL: &str = "$";
const INFO_LOG: &str = "[INFO]";
const ERROR_LOG: &str = "[ERROR]";


fn handle_command(command:&str){
    let command_parts: Vec<&str> = command.split_ascii_whitespace().collect(); 

    match command_parts.as_slice() {
        ["exit"] => {
            println!("{} Exiting the command", INFO_LOG);
            exit(0);
        },
        ["connect", ..] => {
            if command_parts.len() != 2{
                println!("{} Please provide server IP", ERROR_LOG);
                return; 
            }

            // Connect to server command
            let ip = command_parts[1];
            println!("{} Connecting to server at IP {}", INFO_LOG, ip);

        }
        _ => {
            println!("{} Unknown command", ERROR_LOG);
        }
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

        // Handle the command
        handle_command(trimmed_input);

        // Exit condition (optional)
        if trimmed_input == "exit" {
            break;
        }
    }
}
