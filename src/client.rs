use std::{io::{self, Write}, process::{exit, Command}};

const COMMAND_LINE_SYMBOL: &str = "$";
const MESSAGE_LINE_SYMBOL: &str = ">";
const MESSAGE_COMMAND_SYMBOL: &str = "/";
const INFO_LOG: &str = "[INFO]";
const ERROR_LOG: &str = "[ERROR]";


struct WebSocketConnection;


#[derive(Debug, PartialEq, Eq)]
enum WebSocketState{
    Connected,
    Ready, 
}

struct WebSocketClient{
    state: WebSocketState
}

impl WebSocketClient {
    pub fn new() -> Self{
        Self { state: WebSocketState::Ready }
    }

    pub fn get_state(&self) -> &WebSocketState{
        return &self.state; 
    }

    pub fn set_state(&mut self, new_state: WebSocketState){
        self.state = new_state; 
    }
}


fn handle_client_command(command:&str, ws_client: &mut WebSocketClient){
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


            // Do connection

            // Connected!

            // Set state of the client to connected
            ws_client.set_state(WebSocketState::Connected);

        }
        _ => {
            println!("{} Unknown command", ERROR_LOG);
        }
    }

}


fn handle_message_commands(input:&str, ws_client: &mut WebSocketClient){
    // Message command only if the command starts with the command symbol
    // This allows users to execute commands when they are messaging
    if input.starts_with(MESSAGE_COMMAND_SYMBOL){
        // Handle given command
        match input {
            "/disconnect" => {
                println!("{} Disconnecting...", INFO_LOG);
                ws_client.set_state(WebSocketState::Ready);
            },
            _ => {
                println!("{} Unknown command", ERROR_LOG);
            }
        }

    }else{
        // The input is text and should be sent to the server
        // TODO: send message to server
    }
}



#[tokio::main]
async fn main() {
    // Create a new web socket client instance
    let mut ws_client = WebSocketClient::new(); 

    // Infinite input loop
    loop {
        // Print the command line symbol
        if *ws_client.get_state() == WebSocketState::Connected{
            print!("{} ", MESSAGE_LINE_SYMBOL);
        }else{
            print!("{} ", COMMAND_LINE_SYMBOL);
        }

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
        if *ws_client.get_state() == WebSocketState::Connected{
            handle_message_commands(trimmed_input, &mut ws_client);
        }else{
            handle_client_command(trimmed_input, &mut ws_client);
        }
    }
}
