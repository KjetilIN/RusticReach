use serde::{Deserialize, Serialize};

fn default_admin() -> String {
    return "admin".to_string();
}
fn default_server_name() -> String {
    return "MyHostedServer".to_string();
}
fn default_server_description() -> String {
    return "Self hosted server!".to_string();
}

// TODO: use the cargo version of the project instead
fn default_server_version() -> String {
    return "1.0.0".to_string();
}

fn default_max_users() -> usize {
    return 4;
}

fn min_room_count() -> usize{
    return 3; 
}

fn max_user_count_per_room() -> usize{
    return 5;
}

/**
 * Admin config defined in the server config file!
 */
#[derive(Debug, Deserialize, Serialize)]
pub struct AdminConfig {
    #[serde(default = "default_admin")]
    name: String,
    token: String,
    password_hash: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GeneralServerConfig {
    #[serde(default = "default_server_name")]
    server_name: String,

    #[serde(default = "default_server_description")]
    description: String,

    #[serde(default = "default_server_version")]
    server_version: String,

    #[serde(default)]
    welcome_message: Option<String>,

    #[serde(default = "default_max_users")]
    max_user_count: usize,
}


#[derive(Debug, Deserialize, Serialize)]
pub struct RoomConfig{
    #[serde(default = "min_room_count")]
    max_room_count: usize,

    #[serde(default)]
    password_required: bool,

    #[serde(default)]
    allow_room_creation: bool,

    #[serde(default = "max_user_count_per_room")]
    room_capacity: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    admin: AdminConfig,
    general: GeneralServerConfig,
    room: RoomConfig,
}
