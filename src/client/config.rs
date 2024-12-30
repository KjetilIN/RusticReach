use std::{collections::HashMap, fs};

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientConfig {
    user_name: String,
    hash_pass: String,
    user_token: String,

    #[serde(default)]
    validate_server_repo: bool,

    #[serde(default)]
    default_server: Option<DefaultServer>,

    #[serde(default)]
    room_aliases: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DefaultServer {
    server_ip: String,

    #[serde(default)]
    auto_connect: bool,
}

pub fn parse_client_config(file_path: &str) -> Option<ClientConfig> {
    // Try to read to yaml data, if not return none
    let yaml_data = fs::read_to_string(file_path).ok()?;
    return parse_client_config_yml(yaml_data);
}

fn parse_client_config_yml(yaml_data: String) -> Option<ClientConfig> {
    // Parse the client
    let parsed_yaml: Value = serde_yaml::from_str(&yaml_data).unwrap();
    if let Some(client) = parsed_yaml.get("client") {
        let client_data = serde_yaml::from_value(client.clone());
        // If error, return none
        return Some(client_data.ok()?);
    }

    // Was not able to parse the config
    None
}

// Unit test for Client config check
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_config() {
        let yaml = r#"
        client:
          user_name: "zebra123"
          hash_pass: "asfdgfhgdQESHZDJXK"
          user_token: "12345678756432134567"
          validate_server_repo: true
          default_server:
            server_ip: 127.0.0.1
            auto_connect: true
          room_aliases:
            friends: elephant321
            work: anon
            hacker_arena: test
        "#;

        // Parse the input config, should be some
        let result = parse_client_config_yml((&yaml).to_string());
        assert!(result.is_some());
        let config = result.unwrap();

        // All fields are here check
        assert_eq!(config.user_name, "zebra123");
        assert_eq!(config.hash_pass, "asfdgfhgdQESHZDJXK");
        assert_eq!(config.user_token, "12345678756432134567");
        assert!(config.validate_server_repo);

        // Checking the default server is present
        assert!(config.default_server.is_some());
        if let Some(server) = config.default_server {
            assert_eq!(server.server_ip, "127.0.0.1");
            assert_eq!(server.auto_connect, true);
        }

        assert_eq!(
            config.room_aliases.get("friends"),
            Some(&"elephant321".to_string())
        );
    }

    #[test]
    fn test_parse_valid_missing_default_server() {
        let yaml = r#"
        client:
          user_name: "zebra123"
          hash_pass: "asfdgfhgdQESHZDJXK"
          user_token: "12345678756432134567"
          validate_server_repo: true
          room_aliases:
            friends: elephant321
            work: anon
            hacker_arena: test
        "#;

        let result = parse_client_config_yml((&yaml).to_string());

        assert!(result.is_some()); // We expect a valid config to be parsed
        let config = result.unwrap();

        assert_eq!(config.user_name, "zebra123");
        assert_eq!(config.hash_pass, "asfdgfhgdQESHZDJXK");
        assert_eq!(config.user_token, "12345678756432134567");
        assert!(config.validate_server_repo);

        // Checking the default server is not present
        assert!(config.default_server.is_none());
        assert_eq!(
            config.room_aliases.get("friends"),
            Some(&"elephant321".to_string())
        );
    }

    #[test]
    fn test_parse_config_with_missing_optional_fields() {
        let yaml = r#"
        client:
          user_name: "zebra123"
          hash_pass: "asfdgfhgdQESHZDJXK"
          user_token: "12345678756432134567"
          default_server:
            server_ip: 127.0.0.1
        "#;

        // NOTE: default_server is also optional, but if you provide it, then server_ip is required!

        let result = parse_client_config_yml((&yaml).to_string());

        assert!(result.is_some()); // The config should be parsed
        let config = result.unwrap();

        // Auto validate server should be false
        assert_eq!(config.validate_server_repo, false);

        // Check the valid fields are there
        // The default server is still some, but auto connect can not be provided and be false
        assert!(config.default_server.is_some());
        if let Some(server) = config.default_server {
            assert!(!server.server_ip.is_empty());
            assert_eq!(server.auto_connect, false);
        }

        // Missing room_aliases should be an empty HashMap
        assert!(config.room_aliases.is_empty());
    }

    #[test]
    fn test_parse_config_with_missing_required_fields() {
        let yaml = r#"
        client:
          user_name: zebra123
          hash_pass: asfdgfhgdQESHZDJXK
        "#;

        let result = parse_client_config_yml((&yaml).to_string());
        assert!(result.is_none()); // Parsing should fail because of missing required fields
    }

    #[test]
    fn test_parse_empty_yaml() {
        let yaml = r#"
        "#; // Empty YAML file, which is not expected to contain any valid config.

        let result = parse_client_config_yml((&yaml).to_string());
        assert!(result.is_none()); // No valid data in an empty YAML file
    }
}
