#[derive(Debug, Clone)]
pub struct ClientState {
    pub user_name: String,
    pub room: Option<String>,
}

impl ClientState {
    pub fn new(user_name: String, room: Option<String>) -> Self {
        Self {
            user_name: user_name.clone(),
            room: room.clone(),
        }
    }
}
