use std::fs;

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientConfig {
    #[serde(rename = "user-name")]
    user_name: String,

    #[serde(rename = "hash_pass")]
    hash_pass: String,

    #[serde(rename = "user-token")]
    user_token: String,

    #[serde(rename = "validate-server-repo")]
    validate_server_repo: bool,

    #[serde(rename = "default-server")]
    default_server: DefaultServer,

    #[serde(rename = "room_aliases", default)]
    room_aliases: std::collections::HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DefaultServer {
    #[serde(rename = "server-ip")]
    server_ip: String,
    #[serde(rename = "auto-connect")]
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
