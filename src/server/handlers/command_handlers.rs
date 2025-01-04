use crate::{
    core::{
        messages::{Command, ServerMessage},
        user::user::User,
    },
    utils::{hash::hash_str, traits::SendServerReply},
};

pub async fn handle_client_command(command: &Command, current_user: &mut User) {
    // Info log about message
    match command {
        Command::SetName(new_name) => {
            // Changing the name
            current_user.set_user_name(new_name.to_string());

            // Send update message
            let msg = ServerMessage::state_update(&current_user, "New user name set");
            msg.send(&mut current_user.get_session()).await;
        }
        Command::JoinRoom(room) => {
            // Send success message
            let msg = ServerMessage::successful_command("Joined room!");
            msg.send(&mut current_user.get_session()).await;
        }
        Command::LeaveRoom => {
            // Leave room

            // Send update message
            let msg = ServerMessage::state_update(&current_user, "Left room");
            msg.send(&mut current_user.get_session()).await;
        }
        Command::RoomInfo => {
            if let Some(room_name) = current_user.get_room_name() {
                // Find information about the current room
            }
        }
        Command::AuthUser(user_id) => {
            // Set the user id of the user with the given user
            current_user.set_id(hash_str(&user_id));

            // Send auth message back to user
            let msg = ServerMessage::Authenticated;
            msg.send(&mut current_user.get_session()).await;
        }
    }
}
