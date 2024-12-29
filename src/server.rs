use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};
use actix_web::{web, App, HttpRequest, HttpServer, Responder};
use actix_ws::{Message, Session};
use futures_util::{StreamExt as _, SinkExt as _};
use uuid::Uuid;

type Rooms = Arc<Mutex<HashMap<String, HashSet<String>>>>;
type Clients = Arc<Mutex<HashMap<String, Session>>>;

async fn ws(
    req: HttpRequest,
    body: web::Payload,
    rooms: web::Data<Rooms>,
    clients: web::Data<Clients>,
) -> actix_web::Result<impl Responder> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    // Generate a unique client ID for this connection
    let client_id = Uuid::new_v4().to_string();

    // Add the client to the global list of clients
    clients.lock().unwrap().insert(client_id.clone(), session.clone());

    actix_web::rt::spawn({
        let rooms = rooms.clone();
        let clients = clients.clone();

        async move {
            let mut current_room: Option<String> = None;

            while let Some(Ok(msg)) = msg_stream.next().await {
                match msg {
                    Message::Text(text) => {
                        let text = text.trim();

                        // Handle commands
                        if text.starts_with("/join ") {
                            let room_name = text.strip_prefix("/join ").unwrap().to_string();

                            // Leave the current room if necessary
                            if let Some(room) = &current_room {
                                leave_room(&client_id, room, &rooms).await;
                            }

                            // Join the new room
                            join_room(&client_id, &room_name, &rooms).await;
                            current_room = Some(room_name.clone());

                            session.text(format!("Joined room: {}", room_name)).await.unwrap();
                        } else if text == "/leave" {
                            // Leave the current room
                            if let Some(room) = current_room.take() {
                                leave_room(&client_id, &room, &rooms).await;
                                session.text("Left the room").await.unwrap();
                            } else {
                                session.text("You are not in any room").await.unwrap();
                            }
                        } else {
                            // Broadcast messages to the current room
                            if let Some(room) = &current_room {
                                broadcast_message(&client_id, room, &text, &rooms, &clients).await;
                            } else {
                                session.text("Join a room first with /join <room_name>").await.unwrap();
                            }
                        }
                    }

                    Message::Ping(bytes) => {
                        if session.pong(&bytes).await.is_err() {
                            break;
                        }
                    }

                    Message::Close(_) => break,

                    _ => {}
                }
            }

            // Clean up when the client disconnects
            if let Some(room) = &current_room {
                leave_room(&client_id, room, &rooms).await;
            }

            clients.lock().unwrap().remove(&client_id);
        }
    });

    Ok(response)
}

async fn join_room(client_id: &str, room_name: &str, rooms: &web::Data<Rooms>) {
    let mut rooms = rooms.lock().unwrap();
    let room = rooms.entry(room_name.to_string()).or_insert_with(HashSet::new);
    room.insert(client_id.to_string());
}

async fn leave_room(client_id: &str, room_name: &str, rooms: &web::Data<Rooms>) {
    let mut rooms = rooms.lock().unwrap();
    if let Some(room) = rooms.get_mut(room_name) {
        room.remove(client_id);
        if room.is_empty() {
            rooms.remove(room_name); // Clean up empty rooms
        }
    }
}

async fn broadcast_message(
    sender_id: &str,
    room_name: &str,
    message: &str,
    rooms: &web::Data<Rooms>,
    clients: &web::Data<Clients>,
) {
    let rooms = rooms.lock().unwrap();
    let mut clients = clients.lock().unwrap();

    if let Some(room) = rooms.get(room_name) {
        for client_id in room {
            if client_id != sender_id {
                if let Some(session) = clients.get_mut(client_id) {
                    let _ = session.text(format!("{}: {}", sender_id, message)).await;
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let rooms: Rooms = Arc::new(Mutex::new(HashMap::new()));
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(rooms.clone()))
            .app_data(web::Data::new(clients.clone()))
            .route("/ws", web::get().to(ws))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
