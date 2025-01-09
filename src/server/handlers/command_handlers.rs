use crate::{
    core::{
        messages::{Command, ServerMessage},
        room::room::WebRoom,
        user::user::User,
    },
    utils::{hash::hash_str, traits::SendServerReply},
};

async fn join_public_room(
    room_name: String,
    current_user: &mut User,
    server_rooms: &WebRoom,
) -> ServerMessage {
    if let Ok(mut server_rooms_lock) = server_rooms.lock() {
        if server_rooms_lock.is_room_name_taken(room_name.to_string()) {
            // Can join the room only if the room is not password protected
            if let Some(room) = server_rooms_lock.get_room_mut_with_name(room_name.to_string()) {
                if !room.has_password() {
                    if !room.contains_user(&current_user) {
                        // Add the user to the the room
                        match room.add_user(&current_user) {
                            Ok(_) => {
                                // Mutate the state of the user 
                                current_user.set_room(room_name);
                                
                                // Send success message
                                return ServerMessage::successful_command("Joined room!");
                            }
                            Err(err) => {
                                // Send the error message of the room error
                                return ServerMessage::room_error_msg(err);
                            }
                        };
                    } else {
                        // User is already in the room
                        return ServerMessage::room_error_msg(
                            crate::core::room::room::RoomError::UserExists(
                                current_user.get_user_name().to_owned(),
                            ),
                        );
                    }
                }
            }
        }
    }

    return ServerMessage::room_not_found();
}

pub async fn handle_client_command(
    command: &Command,
    current_user: &mut User,
    server_rooms: &WebRoom,
) {
    // Info log about message
    match command {
        Command::SetName(new_name) => {
            // Changing the name
            current_user.set_user_name(new_name.to_string());

            // Send update message
            let msg = ServerMessage::state_update(&current_user, "New user name set");
            msg.send(&mut current_user.get_session()).await;
        }
        Command::JoinPublicRoom(given_room_name) => {
            // Handles joining a public room, and then sends the server message from the action
            join_public_room(given_room_name.to_string(), current_user, server_rooms)
                .await
                .send(&mut current_user.get_session())
                .await;
        }
        Command::LeaveRoom => {
            //TODO: Leave room

            // Send update message
            let msg = ServerMessage::state_update(&current_user, "Left room");
            msg.send(&mut current_user.get_session()).await;
        }
        Command::CreatePublicRoom(room_name) => {
            // Created a room with the given name
            if let Ok(mut rooms) = server_rooms.lock() {
                // Creates a public room
                let res = rooms.create_public_room(room_name.to_string(), 5, &current_user);
                match res {
                    Ok(_) => {
                        // Send OK message back
                        let msg = ServerMessage::created_room(room_name.to_string());
                        msg.send(&mut current_user.get_session()).await;
                    }
                    Err(err) => {
                        // Failed to create the public room
                        let server_msg = ServerMessage::room_error_msg(err);
                        server_msg.send(&mut current_user.get_session()).await;
                    }
                }
            }
        }
        Command::RoomInfo => {
            if let Some(room_name) = current_user.get_room_name() {
                // Find information about the current room
                //TODO: make room info available as a command
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
