use colored::Colorize;
use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    sender: String,
    content: String,
    room: String,
    time_stamp: String
}

impl ChatMessage {
    
    pub fn format(&self) -> String{
        let user_name_color: (u8,u8,u8) = (255,0,140);


        let formatted_time = self.time_stamp.truecolor(211, 211, 211).to_string();
        let formatted_user_name = format!("<{}>", self.sender)
        .truecolor(user_name_color.0, user_name_color.1, user_name_color.2)
        .to_string();

        return format!("{} {} {}", formatted_time, formatted_user_name, self.content)
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
