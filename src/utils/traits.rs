use actix_ws::Session;
use async_trait::async_trait;
use serde::Serialize;

#[async_trait]
pub trait JsonSerializing: Serialize + Send + Sync {
    async fn serialized(&self) -> Option<String> {
        serde_json::to_string(self).ok()
    }
}

#[async_trait]
pub trait SendServerReply: JsonSerializing {
    async fn send(&self, session: &mut Session) {
        match self.serialized().await {
            Some(reply) => {
                // Send the serialized message as a WebSocket text message
                if let Err(err) = session.text(reply).await {
                    eprintln!("Failed to send message: {}", err);
                }
            }
            None => {
                eprintln!("Failed to serialize the struct");
            }
        }
    }
}
