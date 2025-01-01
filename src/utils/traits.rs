use actix_ws::Session;
use serde::Serialize;

use crate::utils::constants::ERROR_LOG;


pub trait JsonSerializing: Serialize{
    async fn serialized(&self) -> Option<String>{
        serde_json::to_string(self).ok()
    }
}


#[async_trait]
pub trait SendServerReply: JsonSerializing {
    async fn send(&self, session: &Session) {
        match self.serialized(){
            Some(reply) => {
                // Send the serialized message as a WebSocket text message
                if let Err(err) = session.text(reply).await {
                    // TODO rollback
                    println!("{} Failed to send message: {}", *ERROR_LOG, err);
                }
            }
            None => {
                println!("Failed to serialize the struct");
            }
        }
    }
}
