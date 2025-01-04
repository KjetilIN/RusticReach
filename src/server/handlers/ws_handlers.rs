use actix_web::web;
use actix_ws::Session;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    core::{room::room::Room, user::user::User},
    utils::constants::INFO_LOG,
};

pub async fn handle_join(room_name: String, user: &mut User, room: &mut Room) {
    // Leave the current room if necessary
    if room.contains_user(user) {
        room.remove_user(user);
    }

    // Log join message
    println!(
        "{} User {} is joining in room: {}",
        *INFO_LOG,
        &user.get_user_name(),
        room_name
    );

    // Join the new room
    let room_join_result = room.add_user(user);
    match room_join_result {
        Ok(_) => (),
        Err(_) => todo!(),
    }
}

pub async fn handle_leave(user: &mut User, room: &mut Room) {
    // Leave the current room
    if room.contains_user(user) {
        room.remove_user(user);
        user.get_session().text("Left the room").await.unwrap();
    } else {
        user.get_session()
            .text("You are not in any room")
            .await
            .unwrap();
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
