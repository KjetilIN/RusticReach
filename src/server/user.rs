use crate::server::room::Rooms;
use actix_web::web;
use actix_ws::Session;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct User {
    user_id: String,
    session: Option<Session>,
    user_name: Option<String>,
    current_room_name: Option<String>,
}

// Set of users for the server
pub type Users = Arc<Mutex<HashMap<String, User>>>;

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

    pub fn get_user_name(&self) -> String {
        self.user_name.clone().unwrap()
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

    pub async fn leave_room(&mut self, user_id: &str, room_name: &str, rooms: &web::Data<Rooms>) {
        // Acquire lock
        let mut rooms = rooms.lock().unwrap();

        // If the room is a valid room!
        if let Some(room) = rooms.get_mut(room_name) {
            // remove the client from the room
            room.remove(user_id);
            self.current_room_name = None;
            if room.is_empty() {
                rooms.remove(room_name);
            }
        }
    }

    pub async fn broadcast_message(
        &self,
        message: &str,
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
                        let _ = client
                            .get_session()
                            .text(format!(
                                "[{}]: {}",
                                self.user_name.clone().unwrap_or("unknown".to_string()),
                                message
                            ))
                            .await;
                    }
                }
            }
        }
    }
}
