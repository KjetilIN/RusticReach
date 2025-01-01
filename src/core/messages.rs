use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::utils::constants::MESSAGE_COMMAND_SYMBOL;

#[derive(Serialize, Deserialize, Clone)]
pub enum ClientMessage {
    Command(Command),
    Chat(ChatMessage),
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Command {
    SetName(String),
    JoinRoom(String),
    LeaveRoom,
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
                if parts.len() == 0 {
                    return Some(Command::LeaveRoom);
                }
            }
            "/name" => {
                if parts.len() == 2 {
                    let new_name = parts[1];
                    return Some(Command::SetName(new_name.to_owned()));
                }
            }
            _ => return None,
        }

        return None;
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    sender: String,
    content: String,
    room: String,
    time_stamp: String,
}

impl ChatMessage {
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
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ServerMessage {
    CommandResult {
        success: bool,
        message: String,
    },
    StateUpdate {
        username: Option<String>,
        current_room: Option<String>,
    },
    ChatMessage,
}
