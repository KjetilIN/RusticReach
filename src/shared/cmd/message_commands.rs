use super::{command::Command, commands::Commands};
use lazy_static::lazy_static;

pub fn setup() -> Commands {
    let join_command = Command::new("/join")
        .description("Join a given room on the server")
        .usage("/join <room-name>");

    let disconnect_command = Command::new("/disconnect")
        .description("Disconnect from the server")
        .usage("/disconnect");

    let leave_command = Command::new("/leave")
        .description("Leave the room that you currently are in")
        .usage("/leave");

    let name_command = Command::new("/name")
        .description("Sets the room name")
        .usage("/name <name>");

    // Create server list of commands
    let mut server_commands = Commands::new();
    server_commands.push_command(join_command);
    server_commands.push_command(disconnect_command);
    server_commands.push_command(leave_command);
    server_commands.push_command(name_command);

    // Return the server commands
    server_commands
}

// Lazy static load the message commands
lazy_static! {
    pub static ref MESSAGE_COMMANDS: Commands = setup();
}
