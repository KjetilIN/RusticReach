use super::user::User;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Room {
    id: String,
    name: String,
    capacity: usize,
    joined_users: HashMap<String, String>,
}

#[derive(Debug)]
pub enum RoomError {
    MaxRoomCountReached,
    MaxCapacityReached,
    NameOccupied,
    InvalidAction(String),
}

impl Room {
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
}

/// Collection of Rooms that the server currently has
#[derive(Debug)]
pub struct ServerRooms {
    rooms: HashMap<String, Room>,
    max_rooms_count: usize,
}

impl ServerRooms {
    pub fn with_max_room_count(max: usize) -> Self {
        Self {
            max_rooms_count: max,
            rooms: HashMap::with_capacity(max),
        }
    }

    fn room_name_not_taken(&self, room_name: String) -> bool {
        for (_, room) in &self.rooms {
            if room.name() == room_name {
                return true;
            }
        }
        false
    }

    pub fn create_room(&self, room_name: String) -> Result<(), RoomError> {
        // Create room only if we are allowed to create more rooms
        if self.rooms.len() < self.max_rooms_count {
            // Create room
            if !self.room_name_not_taken(room_name) {
                return Ok(());
            }
            // The given room name is taken for this server
            return Err(RoomError::NameOccupied);
        }
        return Err(RoomError::MaxRoomCountReached);
    }
}
