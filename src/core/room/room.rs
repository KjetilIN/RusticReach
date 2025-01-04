use serde::{Deserialize, Serialize};
use uuid::Uuid;

use std::collections::HashMap;

use crate::{core::user::user::User, utils::hash::hash_str};

/// Represents any type of error that a user might have had interacting with a Room in some way
#[derive(Debug)]
pub enum RoomError {
    MaxRoomCount(usize),
    MaxCapacityReached,
    NameOccupied,
    InvalidAction(String),
}

/// Represents a state of a given room at the given time
#[derive(Debug)]
pub struct Room {
    id: String,
    owner_id: String,
    name: String,
    capacity: usize,
    joined_users: HashMap<String, String>,
    password_hash: Option<String>,
}

impl Room {
    /// Create a new room with random id
    pub fn new(owner: &User, room_name: String, capacity: usize) -> Self{
        let room_id = Uuid::new_v4().to_string();
        Self { id: room_id, owner_id: owner.get_id().to_string(), name: room_name, capacity, joined_users: HashMap::new(), password_hash: None }
    }

    /// Sets the password of the room
    /// Takes the password in plain text and hashes it before storing
    pub fn password(mut self, plain_password: String) -> Self{
        self.password_hash = Some(hash_str(&plain_password));
        self
    }

    /// Returns true if the room requires a password
    pub fn has_password(&self) -> bool{
        self.password_hash.is_some()
    }

    /// Authenticate method
    /// 
    /// If no password is set, then always returns true
    pub fn is_correct_password(&self, input: &str) -> bool{
        if self.has_password(){
            let hashed_input = hash_str(input);
            return hashed_input == self.password_hash.clone().unwrap()
        }

        // No password, return true
        true
    }


    /// Check if the given user is the owner of the given room room
    pub fn is_owned_by(&self, user: &User) -> bool{
        return self.owner_id == user.get_id()
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
        self.joined_users.len()
    }

    /// Remove the given user from the room
    pub fn remove_user(&mut self, user: &User) {
        if self.joined_users.contains_key(user.get_id()) {
            self.joined_users.remove(user.get_id());
        }
    }

    /// Add a user to the list of joined users
    pub fn add_user(&mut self, user: &User) -> Result<(), RoomError> {
        if self.capacity > 0 {
            if !self.joined_users.contains_key(user.get_id()) {
                self.joined_users
                    .insert(user.get_id().to_owned(), user.get_user_name().to_owned());
            }
            Ok(())
        } else {
            return Err(RoomError::MaxCapacityReached);
        }
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
#[derive(Debug)]
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
    fn room_name_taken(&self, room_name: String) -> bool {
        for (_, room) in &self.rooms {
            if room.name() == room_name {
                return true;
            }
        }
        false
    }

    /// Get the given room id
    fn get_room_id(&self, room_name: String) -> Option<String> {
        for (_, room) in &self.rooms {
            if room.name() == room_name {
                return Some(room.id.clone());
            }
        }
        None
    }

    /// Create a new password protected room
    /// 
    /// Uses the room configuration to do the allowed operations
    pub fn create_private_room(&mut self, room_name: String, room_capacity:usize, owner: &User, password: String) -> Result<(), RoomError> {
        // TODO: get from config the privileges of room creation


        // Create room only if we are allowed to create more rooms
        if self.rooms.len() < self.max_rooms_count {
            // Create room
            if !self.room_name_taken(room_name.clone()) {
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
    pub fn create_public_room(&mut self, room_name: String, room_capacity:usize, owner: &User) -> Result<(), RoomError> {
        // TODO: get from config the privileges of room creation


        // Create room only if we are allowed to create more rooms
        if self.rooms.len() < self.max_rooms_count {
            // Create room
            if !self.room_name_taken(room_name.clone()) {
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
