use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::{
    client::state::ClientState,
    utils::{
        constants::{MESSAGE_COMMAND_SYMBOL, SELF_USER},
        time::get_time_string,
        traits::{JsonSerializing, SendServerReply},
    },
};

use super::user::user::User;

#[derive(Serialize, Deserialize, Clone)]
pub enum ClientMessage {
    Command(Command),
    Chat(ChatMessage),
}

impl JsonSerializing for ClientMessage {}
impl SendServerReply for ClientMessage {}

#[derive(Serialize, Deserialize, Clone)]
pub enum Command {
    SetName(String),
    JoinRoom(String),
    LeaveRoom,
    RoomInfo,
    AuthUser(String),
}

impl Command {
    pub fn from_str(input: &str) -> Option<Self> {
        if input.is_empty() || !input.starts_with(MESSAGE_COMMAND_SYMBOL) {
            return None;
        }

        // Split the input into different parts
        let parts: Vec<&str> = input.split_ascii_whitespace().collect();

        // Based on the first part, we have to parse the command
        match parts[0] {
            "/join" => {
                if parts.len() == 2 {
                    let room_name = parts[1];
                    return Some(Command::JoinRoom(room_name.to_owned()));
                }
            }
            "/leave" => {
                if parts.len() == 1 {
                    return Some(Command::LeaveRoom);
                }
            }
            "/name" => {
                if parts.len() == 2 {
                    let new_name = parts[1];
                    return Some(Command::SetName(new_name.to_owned()));
                }
            }
            "/room" => {
                if parts.len() == 1 {
                    return Some(Command::RoomInfo);
                }
            }
            _ => return None,
        }

        return None;
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatMessage {
    pub sender: String,
    content: String,
    pub room: String,
    time_stamp: String,
}

impl ChatMessage {
    pub fn create(client_state: &ClientState, message_content: String) -> Result<Self, ()> {
        if client_state.room.is_none() {
            return Err(());
        }
        Ok(Self {
            sender: client_state.user_name.clone(),
            content: message_content,
            room: client_state.room.clone().unwrap(),
            time_stamp: get_time_string(),
        })
    }

    pub fn format(&self) -> String {
        let user_name_color: (u8, u8, u8) = (255, 0, 140);

        let formatted_time = self.time_stamp.truecolor(211, 211, 211).to_string();
        let formatted_user_name = format!("<{}>", self.sender)
            .truecolor(user_name_color.0, user_name_color.1, user_name_color.2)
            .to_string();

        return format!(
            "{} {} {}",
            formatted_time, formatted_user_name, self.content
        );
    }

    pub fn format_self(&self) -> String {
        let formatted_time = self.time_stamp.truecolor(211, 211, 211).to_string();
        let formatted_user_name = (*SELF_USER).yellow().to_string();

        return format!(
            "{} {} {}",
            formatted_time, formatted_user_name, self.content
        );
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommandResult {
    success: bool,
    message: String,
}

impl JsonSerializing for CommandResult {}
impl SendServerReply for CommandResult {}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerMessage {
    CommandResult {
        success: bool,
        message: String,
    },
    StateUpdate {
        username: Option<String>,
        current_room: Option<String>,
        message: String,
    },
    Chat(ChatMessage),
    Authenticated,
}

impl ServerMessage {
    pub fn failed_command(message: &str) -> Self {
        return Self::CommandResult {
            success: false,
            message: message.to_string(),
        };
    }

    pub fn successful_command(message: &str) -> Self {
        return Self::CommandResult {
            success: true,
            message: message.to_string(),
        };
    }

    pub fn from_chat_msg(chat_message: ChatMessage) -> Self {
        return Self::Chat(chat_message);
    }

    pub fn state_update(user: &User, message: &str) -> Self {
        Self::StateUpdate {
            username: Some(user.get_user_name().to_owned()),
            current_room: user.get_room_name(),
            message: message.to_string(),
        }
    }
}

impl JsonSerializing for ServerMessage {}
impl SendServerReply for ServerMessage {}
