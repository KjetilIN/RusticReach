use actix_web::web;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use std::{
    collections::{hash_map::Values, HashMap},
    fmt::format,
    sync::{Arc, Mutex},
};

use crate::{
    core::user::user::User,
    utils::{constants::SERVER_INFO, hash::hash_str},
};

/// Represents any type of error that a user might have had interacting with a Room in some way
#[derive(Debug)]
pub enum RoomError {
    /// Given server has met the max capacity of rooms
    MaxRoomCount(usize),
    MaxCapacityReached,
    NameOccupied,

    // User is already in the room
    UserExists(String),

    /// Action was not available because of the given reason
    InvalidAction(String),
    PasswordRequired,

    RoomNotFound,
}

impl RoomError {
    /// Returns a formatted message from the room error
    pub fn message(&self) -> String {
        match self {
            RoomError::MaxRoomCount(count) => format!(
                "{} {}/{} rooms created. No more available rooms available...",
                *SERVER_INFO, count, count
            ),
            RoomError::MaxCapacityReached => format!("{} Room is full", *SERVER_INFO),
            RoomError::NameOccupied => format!("{} Room already exists", *SERVER_INFO),
            RoomError::InvalidAction(msg) => format!("{} {}", *SERVER_INFO, msg),
            RoomError::PasswordRequired => format!("{} Room is password protected", *SERVER_INFO),
            RoomError::RoomNotFound => format!("{} Room does not exists", *SERVER_INFO),
            RoomError::UserExists(user_name) => format!(
                "{} User '{}' already exist in the room. Please change username",
                *SERVER_INFO, user_name
            ),
        }
    }
}

/// Represents a state of a given room at the given time
pub struct Room {
    id: String,
    owner_id: String,
    name: String,
    capacity: usize,
    users: HashMap<String, User>,
    password_hash: Option<String>,
}

/// Represent a application level data of the struct ServerRooms, protected by a mutex
pub type WebRoom = web::Data<Arc<Mutex<ServerRooms>>>;

impl Room {
    /// Create a new room with random id
    pub fn new(owner: &User, room_name: String, capacity: usize) -> Self {
        let room_id = Uuid::new_v4().to_string();
        Self {
            id: room_id,
            owner_id: owner
                .get_id()
                .expect("Owner of group had an ID that was not set")
                .to_string(),
            name: room_name,
            capacity,
            users: HashMap::new(),
            password_hash: None,
        }
    }

    /// Sets the password of the room
    /// Takes the password in plain text and hashes it before storing
    pub fn password(mut self, plain_password: String) -> Self {
        self.password_hash = Some(hash_str(&plain_password));
        self
    }

    /// Returns true if the room requires a password
    pub fn has_password(&self) -> bool {
        self.password_hash.is_some()
    }

    /// Authenticate method
    ///
    /// If no password is set, then always returns true
    pub fn is_correct_password(&self, input: &str) -> bool {
        if self.has_password() {
            let hashed_input = hash_str(input);
            return hashed_input == self.password_hash.clone().unwrap();
        }

        // No password, return true
        true
    }

    /// Check if the given user is the owner of the given room room
    pub fn is_owned_by(&self, user: &User) -> bool {
        if user.get_id().is_none() {
            return false;
        }
        return self.owner_id == user.get_id().unwrap();
    }

    pub fn contains_user(&self, user: &User) -> bool {
        assert!(
            user.get_id().is_some(),
            "contains_user, id not set for the user"
        );
        for current_user in self.users.values() {
            if user.get_id().unwrap() == current_user.get_id().unwrap() {
                return true;
            }
        }

        false
    }

    /// Gets the name of the user
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Get the amount of free spaces within a room
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get the amount of users that are in the room
    pub fn joined_user_count(&self) -> usize {
        self.users.len()
    }

    /// Remove the given user from the room
    pub fn remove_user(&mut self, user: &User) {
        if let Some(user_id) = user.get_id() {
            if self.users.contains_key(user_id) {
                self.users.remove(user_id);
            }
        }
    }

    /// Add a user to the list of joined users
    ///
    /// The method clones the user, changes the room of the user, and adds it to the room
    pub fn add_user(&mut self, user: &User) -> Result<(), RoomError> {
        if self.capacity > 0 {
            if let Some(user_id) = user.get_id() {
                if !self.users.contains_key(user_id) {
                    // Make the user mutable, change the room name, and then insert it to the room
                    self.users.insert(user_id.to_owned(), user.clone());
                }
            } else {
                // Should not be possible, panic
                //TODO: handle unexpected behavior better
                panic!("User did not have an ID")
            }
            Ok(())
        } else {
            return Err(RoomError::MaxCapacityReached);
        }
    }

    /// Returns an iterator of all Users in the room
    pub fn iter_users(&self) -> Values<'_, String, User> {
        self.users.values()
    }

    /// Returns a struct that represents the information about the current room
    pub fn info(&self) -> RoomInformation {
        unimplemented!()
    }
}

/// Room information that any user without any privileges can see about the Room the user currently is in
#[derive(Serialize, Deserialize, Clone)]
pub struct RoomInformation {
    users_count: usize,
    room_size: usize,
    room_owner_username: String,
    current_users: Vec<String>,
}

/// Collection of Rooms that the server currently has
pub struct ServerRooms {
    rooms: HashMap<String, Room>,
    max_rooms_count: usize,
}

impl ServerRooms {
    /// Create a new list of server rooms with a given max count for amount of rooms allowed on the server
    pub fn with_max_room_count(max: usize) -> Self {
        Self {
            max_rooms_count: max,
            rooms: HashMap::with_capacity(max),
        }
    }

    /// Checks if the given room is already exists on the server
    pub fn is_room_name_taken(&self, room_name: String) -> bool {
        for (_, room) in &self.rooms {
            if room.name() == room_name {
                return true;
            }
        }
        false
    }

    /// Get the given room id
    pub fn get_room_id(&self, room_name: String) -> Option<String> {
        for (_, room) in &self.rooms {
            if room.name() == room_name {
                return Some(room.id.clone());
            }
        }
        None
    }

    /// Returns a mutable reference to the room of the user
    pub fn get_room_mut(&mut self, user: &User) -> Option<&mut Room> {
        let room_name = user.get_room_name()?;
        if let Some(room) = self.rooms.get_mut(&room_name) {
            return Some(room);
        }

        // User is not in the room
        None
    }

    /// Returns a reference to the room that the user is in
    pub fn get_room(&self, user: &User) -> Option<&Room> {
        let room_name = user.get_room_name()?;
        if let Some(room) = self.rooms.get(&room_name) {
            return Some(room);
        }

        // User is not in the room
        None
    }

    pub fn get_room_mut_with_name(&mut self, room_name: String) -> Option<&mut Room> {
        self.rooms.get_mut(&room_name)
    }

    /// Get the room with given name
    pub fn get_room_with_name(&self, room_name: String) -> Option<&Room> {
        self.rooms.get(&room_name)
    }

    /// Create a new password protected room
    ///
    /// Uses the room configuration to do the allowed operations
    pub fn create_private_room(
        &mut self,
        room_name: String,
        room_capacity: usize,
        owner: &User,
        password: String,
    ) -> Result<(), RoomError> {
        // TODO: get from config the privileges of room creation

        // Create room only if we are allowed to create more rooms
        if self.rooms.len() < self.max_rooms_count {
            // Create room
            if !self.is_room_name_taken(room_name.clone()) {
                // Create a new room with password and insert it to the list of rooms
                let room = Room::new(owner, room_name.clone(), room_capacity).password(password);
                self.rooms.insert(room_name, room);
                return Ok(());
            }
            // The given room name is taken for this server
            return Err(RoomError::NameOccupied);
        }
        return Err(RoomError::MaxRoomCount(self.rooms.len()));
    }

    /// Create a public room without password
    pub fn create_public_room(
        &mut self,
        room_name: String,
        room_capacity: usize,
        owner: &User,
    ) -> Result<(), RoomError> {
        // TODO: get from config the privileges of room creation

        // Create room only if we are allowed to create more rooms
        if self.rooms.len() < self.max_rooms_count {
            // Create room
            if !self.is_room_name_taken(room_name.clone()) {
                // Create a new room with password and insert it to the list of rooms
                let room = Room::new(owner, room_name.clone(), room_capacity);
                self.rooms.insert(room_name, room);
                return Ok(());
            }
            // The given room name is taken for this server
            return Err(RoomError::NameOccupied);
        }
        return Err(RoomError::MaxRoomCount(self.rooms.len()));
    }

    /// Delete the given room name from the server
    pub fn delete_room(&mut self, room_name: String) {
        if let Some(room_id) = self.get_room_id(room_name) {
            self.rooms.remove(&room_id);
        }
    }
}
