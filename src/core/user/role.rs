pub enum UserRole {
    /// Creator of the chat server (hosts the chat server)
    ServerAdmin,

    /// Only access to certain action for the given room
    /// Given string is the room id, which should only be available for the user when user can do actions
    RoomAdmin(String),

    //TODO: role that contains actions, which can be given by server admin

    /// Regular user, no special role
    Regular,
}

impl Default for UserRole {
    /// Creates a user role with regular
    fn default() -> Self {
        return UserRole::Regular;
    }
}

impl UserRole {
    /// Get the protection ring value.
    ///
    /// The lower the value, the more privilege does the user have
    /// Reference: https://en.wikipedia.org/wiki/Protection_ring
    pub fn protection_ring_value(&self) -> usize {
        return match &self {
            UserRole::ServerAdmin => 0,
            UserRole::RoomAdmin(_) => 1,
            UserRole::Regular => 2,
        };
    }
}
