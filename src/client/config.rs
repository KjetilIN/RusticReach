use std::{collections::HashMap, fs};

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientConfig {
    user_name: String,
    hash_pass: String,
    user_token: String,
    validate_server_repo: bool,
    default_server: DefaultServer,
    room_aliases: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DefaultServer {
    server_ip: String,
    auto_connect: bool,
}

pub fn parse_client_config(file_path: &str) -> Option<ClientConfig> {
    // Try to read to yaml data, if not return none
    let yaml_data = fs::read_to_string(file_path).ok()?;

    // Parse the client
    let parsed_yaml: Value = serde_yaml::from_str(&yaml_data).unwrap();
    if let Some(client) = parsed_yaml.get("client") {
        let client_data: ClientConfig = serde_yaml::from_value(client.clone()).unwrap();
        return Some(client_data);
    }

    // Was not able to parse the config
    None
}
