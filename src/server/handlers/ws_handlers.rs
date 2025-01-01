use actix_web::web;
use actix_ws::Session;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    core::{messages::ServerMessage, user::User},
    utils::{constants::ERROR_LOG, traits::JsonSerializing},
};

pub type WebRoom = web::Data<Arc<Mutex<HashMap<String, std::collections::HashSet<String>>>>>;

pub async fn handle_join(
    session: &mut Session,
    room_name: String,
    user: &mut User,
    user_id: &String,
    rooms: &WebRoom,
) {
    // Leave the current room if necessary
    if user.has_joined_room() {
        user.leave_room(&user_id, &rooms).await;
    }

    // Log join message
    println!(
        "[INFO] User {} is joining in room: {}",
        &user.get_user_name(),
        room_name
    );

    // Join the new room
    user.join_room(&user_id, &room_name, &rooms).await;
}

pub async fn handle_leave(
    session: &mut Session,
    current_room: &mut Option<String>,
    user: &mut User,
    user_id: &String,
    rooms: &WebRoom,
) {
    // Leave the current room
    if let Some(_) = current_room.take() {
        user.leave_room(&user_id, &rooms).await;
        session.text("Left the room").await.unwrap();
    } else {
        session.text("You are not in any room").await.unwrap();
    }
}

pub async fn handle_name(session: &mut Session, text: String, user: &mut User) {
    let input: Vec<&str> = text.split_ascii_whitespace().into_iter().collect();
    if input.len() == 2 {
        let user_name = input[1];
        if user_name.is_empty() || user_name.len() < 3 {
            // Invalid username
            session
                .text("User name must be at least 3 chars long")
                .await
                .unwrap();
        } else {
            // Set the user name
            println!("Renamed user: {} to {}", user.get_user_name(), user_name);
            user.set_user_name(user_name.to_owned());
        }
    } else {
        // Invalid use of the command
        session
            .text("Set username with /name <user-name>")
            .await
            .unwrap();
    }
}
