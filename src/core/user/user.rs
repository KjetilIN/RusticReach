use actix_ws::Session;

use crate::{
    core::{messages::ServerMessage, room::room::Room},
    utils::traits::SendServerReply,
};

use super::role::UserRole;

#[derive(Clone)]
pub struct User {
    // If the ID has not been set, then the user are not validated by the server
    id: Option<String>,
    name: Option<String>,
    role: UserRole,
    room_name: Option<String>,
    session: Option<Session>,
}

impl User {
    /// Creates a new user from the given Session
    pub fn new(session: Session) -> Self {
        Self {
            id: None,
            room_name: None,
            name: None,
            role: UserRole::default(),
            session: Some(session),
        }
    }

    pub fn set_user_name(&mut self, user_name: String) {
        self.name = Some(user_name);
    }

    pub fn has_joined_room(&self) -> bool {
        self.room_name.is_some()
    }

    pub fn get_session(&self) -> Session {
        self.session.clone().unwrap()
    }

    pub fn get_user_name(&self) -> &str {
        if let Some(name) = &self.name {
            return &name;
        }

        return "unknown";
    }

    /// Set the id of the user
    ///
    /// The id can only be set once. Once it is set, we cannot set it again
    pub fn set_id(&mut self, id: String) {
        if self.id.is_none() {
            self.id = Some(id);
        }
    }

    pub fn get_role(&self) -> &UserRole {
        &self.role
    }

    pub fn get_id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    pub fn get_room_name(&self) -> Option<String> {
        //TODO: improve this, should not need to clone
        self.room_name.clone()
    }

    pub fn take_room(&mut self) -> Option<String> {
        self.room_name.take()
    }

    pub fn set_room(&mut self, room_name: String) {
        self.room_name = Some(room_name);
    }

    pub async fn broadcast_message(&self, message: &ServerMessage, room: &Room) {
        // Client must have a room
        assert!(
            self.room_name.is_some(),
            "Client tried to cast a message when room was none"
        );

        // ID must be set before being able to talk in the group
        assert!(
            self.id.is_some(),
            "ID of the user was None, when broadcast was tried"
        );

        // Make sure that the user himself is in the room that will broadcast the message in the room
        assert!(
            room.contains_user(self),
            "User was not in the room as expected"
        );

        for user in room.iter_users() {
            message.send(&mut user.get_session().clone()).await;
        }
    }
}
