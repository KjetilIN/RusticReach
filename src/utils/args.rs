use std::env;

use crate::{
    client::config::{parse_client_config, ClientConfig},
    utils::constants::ERROR_LOG,
};

/**
 * Validates the input arguments of the
 *
 * Logs error and info
 *
 * Returns either the parsed client config or an error.
 */
pub fn validate_args() -> Result<ClientConfig, String> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 || &args[1] != "-c" || args[2].is_empty() {
        let message = format!("{} Usage: <program> -c <client.yml>", *ERROR_LOG);
        return Err(message);
    }

    let file_path = &args[2];
    if let Some(config) = parse_client_config(&file_path) {
        return Ok(config);
    } else {
        return Err(format!(
            "{} Provided client config {} could not be parsed",
            *ERROR_LOG, file_path
        ));
    }
}
