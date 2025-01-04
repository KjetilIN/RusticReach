use tokio_stream::StreamExt;

use crate::{
    client::config::ClientConfig,
    core::{
        messages::{ClientMessage, ServerMessage},
        user::user::User,
    },
    utils::{
        constants::{INFO_LOG, WARNING_LOG},
        hash::hash_str,
        terminal_ui::TerminalUI,
        traits::SendServerReply,
    },
};

use super::runtime::WsFramedStream;

/// Authenticate the given user
///
/// Solve auth by making the user send its hashed token over to the server.
/// The server will then store this token for the given user instance on server side.
/// Adds information to the terminal UI, or returns Err(String) when there is an error, that should not have happened
///
/// Returns Ok(()) when the user is authenticated
pub async fn auth_user(
    user: &mut User,
    config: &ClientConfig,
    terminal_ui: &mut TerminalUI,
    stream: &mut WsFramedStream,
) -> Result<(), String> {
    // Hash the user token and then send auth message to server
    let hashed_token = hash_str(config.get_token());
    let auth_msg = ClientMessage::Command(crate::core::messages::Command::AuthUser(hashed_token));
    auth_msg.send(&mut user.get_session()).await;

    // Wait until received auth
    let wait_msg = format!("{} Waiting for server to auth user...", *INFO_LOG);
    terminal_ui.add_message(wait_msg);

    // Read from websocket stream until we get a response back
    while let Some(Ok(msg)) = stream.next().await {
        match msg {
            awc::ws::Frame::Text(bytes) => {
                match String::from_utf8(bytes.to_vec()) {
                    Ok(valid_str) => {
                        // Parse server message, or ignore the message
                        let server_msg: ServerMessage = match serde_json::from_str(&valid_str) {
                            Ok(msg) => msg,
                            Err(err) => {
                                return Err(format!(
                                    "Failed to parse WebSocket message as UTF-8: {}",
                                    err
                                ));
                            }
                        };

                        // Only care about the authenticated message
                        match server_msg {
                            ServerMessage::Authenticated => {
                                // The user has been authenticated!
                                terminal_ui.add_message(format!(
                                    "{} User successfully authenticated!",
                                    *INFO_LOG
                                ));
                                return Ok(());
                            }
                            _ => {
                                terminal_ui.add_message(format!(
                                    "{} Ignored unrecognized message: {}",
                                    *WARNING_LOG, valid_str
                                ));
                                continue;
                            }
                        }
                    }
                    Err(_) => todo!(),
                }
            }
            awc::ws::Frame::Close(reason) => {
                return Err(format!(
                    "WebSocket connection closed during authentication: {:?}",
                    reason
                ));
            }
            _ => {
                terminal_ui.add_message(format!("{} Ignored unrecognized message", *WARNING_LOG));
                continue;
            }
        }
    }

    return Err("An unknown error happen during auth".to_string());
}