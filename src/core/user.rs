use actix_web::web;
use actix_ws::Session;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use crate::{
    core::messages::ServerMessage,
    server::room::Rooms,
    utils::traits::{JsonSerializing, SendServerReply},
};

use super::messages::ChatMessage;

// Set of users for the server
pub type Users = Arc<Mutex<HashMap<String, User>>>;

#[derive(Clone)]
pub struct User {
    user_id: String,
    session: Option<Session>,
    user_name: Option<String>,
    current_room_name: Option<String>,
}

impl User {
    pub fn new(user_id: String, session: Session) -> Self {
        Self {
            user_id,
            current_room_name: None,
            user_name: None,
            session: Some(session),
        }
    }

    pub fn set_user_name(&mut self, user_name: String) {
        self.user_name = Some(user_name);
    }

    pub fn has_joined_room(&self) -> bool {
        self.current_room_name.is_some()
    }

    pub fn get_session(&self) -> Session {
        self.session.clone().unwrap()
    }

    pub fn get_user_name(&self) -> &str {
        if let Some(name) = &self.user_name {
            return &name;
        }

        return "unknown";
    }

    pub fn get_id(&self) -> &str {
        &self.user_id
    }

    pub fn get_room_name(&self) -> Option<String> {
        //TODO: improve this, should not need to clone
        self.current_room_name.clone()
    }

    pub fn take_room(&mut self) -> Option<String> {
        self.current_room_name.take()
    }

    pub fn set_room(&mut self, room_name: String) {
        self.current_room_name = Some(room_name);
    }

    pub async fn join_room(&mut self, user_id: &str, room_name: &str, rooms: &web::Data<Rooms>) {
        // Set the room from the rooms
        let mut rooms = rooms.lock().unwrap();
        let room = rooms
            .entry(room_name.to_string())
            .or_insert_with(HashSet::new);
        room.insert(user_id.to_string());

        // Set the current room name to the client
        self.current_room_name = Some(room_name.to_owned());
    }

    pub async fn leave_room(&mut self, user_id: &str, rooms: &web::Data<Rooms>) {
        // Acquire lock
        let mut rooms = rooms.lock().unwrap();
        if let Some(current_room) = &self.current_room_name.clone() {
            // If the room is a valid room!
            if let Some(room) = rooms.get_mut(current_room) {
                // Remove the client from the room
                room.remove(user_id);

                self.current_room_name = None;

                // Remove the room if there is no user here
                // TODO: make this optional
                if room.is_empty() {
                    rooms.remove(current_room);
                }
            }
        }
    }

    pub async fn broadcast_message(
        &self,
        message: &ServerMessage,
        rooms: &web::Data<Rooms>,
        users: &web::Data<Users>,
    ) {
        // Client must have a room
        assert!(
            self.current_room_name.is_some(),
            "Client tried to cast a message when room was none"
        );

        let rooms = rooms.lock().unwrap();
        let mut users = users.lock().unwrap();

        if let Some(room) = rooms.get(&self.current_room_name.clone().unwrap()) {
            for user_id in room {
                if *user_id != self.user_id {
                    if let Some(client) = users.get_mut(user_id) {
                        message.send(&mut client.get_session()).await;
                    }
                }
            }
        }
    }
}
