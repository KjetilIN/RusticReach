use std::{collections::HashMap, fs};

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientConfig {
    pub user_name: String,
    pub hash_pass: String,
    pub user_token: String,
    pub validate_server_repo: bool,
    pub default_server: DefaultServer,
    pub room_aliases: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DefaultServer {
    pub server_ip: String,
    pub auto_connect: bool,
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


// Unit test for Client config check
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    // Helper function to create a temporary YAML file for testing
    fn create_test_yaml(content: &str) -> String {
        let temp_file = "test_config.yaml";
        let mut file = File::create(temp_file).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        temp_file.to_string()
    }

    #[test]
    fn test_parse_valid_config() {
        let yaml = r#"
        client:
          user_name: zebra123
          hash_pass: asfdgfhgdQESHZDJXK
          user_token: 12345678756432134567
          validate_server_repo: true
          default_server:
            server_ip: 127.0.0.1
            auto_connect: true
          room_aliases:
            friends: elephant321
            work: anon
            hacker_arena: test
        "#;

        let file_path = create_test_yaml(yaml);
        let result = parse_client_config(&file_path);

        assert!(result.is_some()); // We expect a valid config to be parsed
        let config = result.unwrap();

        assert_eq!(config.user_name, "zebra123");
        assert_eq!(config.hash_pass, "asfdgfhgdQESHZDJXK");
        assert_eq!(config.user_token, "12345678756432134567");
        assert!(config.validate_server_repo);
        assert_eq!(config.default_server.server_ip, "127.0.0.1");
        assert!(config.default_server.auto_connect);
        assert_eq!(config.room_aliases.get("friends"), Some(&"elephant321".to_string()));
    }

    #[test]
    fn test_parse_config_with_missing_optional_fields() {
        let yaml = r#"
        client:
          user_name: zebra123
          hash_pass: asfdgfhgdQESHZDJXK
          user_token: 12345678756432134567
          validate_server_repo: true
          default_server:
            server_ip: 127.0.0.1
            auto_connect: true
        "#;

        let file_path = create_test_yaml(yaml);
        let result = parse_client_config(&file_path);

        assert!(result.is_some()); // The config should be parsed
        let config = result.unwrap();

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

        let file_path = create_test_yaml(yaml);
        let result = parse_client_config(&file_path);

        assert!(result.is_none()); // Parsing should fail because of missing required fields
    }

    #[test]
    fn test_parse_malformed_yaml() {
        let yaml = r#"
        client:
          user_name: zebra123
          hash_pass: asfdgfhgdQESHZDJXK
          user_token: 12345678756432134567
          validate_server_repo: true
          default_server:
            server_ip: 127.0.0.1
            auto_connect: true
          room_aliases:
            friends: elephant321
            work: anon
            hacker_arena: test
        "#; // Missing closing brackets or extra indentation could make this malformed

        let file_path = create_test_yaml(yaml);
        let result = parse_client_config(&file_path);

        assert!(result.is_none()); // The config is malformed and should return None
    }

    #[test]
    fn test_parse_empty_yaml() {
        let yaml = r#"
        "#; // Empty YAML file, which is not expected to contain any valid config.

        let file_path = create_test_yaml(yaml);
        let result = parse_client_config(&file_path);

        assert!(result.is_none()); // No valid data in an empty YAML file
    }
}