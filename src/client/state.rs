#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ClientState {
    id: String,
    pub user_name: String,
    pub room: Option<String>,
}

impl ClientState {
    pub fn new(id: String, user_name: String, room: Option<String>) -> Self {
        Self {
            id,
            user_name: user_name.clone(),
            room: room.clone(),
        }
    }
}
