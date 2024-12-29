use actix_web::{web, App, HttpRequest, HttpServer, Responder};
use actix_ws::{Message, Session};
use futures_util::StreamExt as _;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};
use uuid::Uuid;

#[derive(Clone)]
struct Client {
    client_id: String,
    session: Option<Session>,
    user_name: Option<String>,
    current_room_name: Option<String>,
}

type Rooms = Arc<Mutex<HashMap<String, HashSet<String>>>>;
type Clients = Arc<Mutex<HashMap<String, Client>>>;

impl Client {
    pub fn new(client_id: String, session: Session) -> Self {
        Self {
            client_id,
            current_room_name: None,
            user_name: None,
            session: Some(session),
        }
    }

    pub fn set_user_name(&mut self, user_name: String) {
        self.user_name = Some(user_name);
    }

    pub fn has_joined_room(&self) -> bool {
        self.current_room_name.is_some()
    }

    pub fn get_session(&self) -> Session {
        self.session.clone().unwrap()
    }

    pub fn get_user_name(&self) -> String {
        self.user_name.clone().unwrap()
    }

    async fn join_room(&mut self, client_id: &str, room_name: &str, rooms: &web::Data<Rooms>) {
        // Set the room from the rooms
        let mut rooms = rooms.lock().unwrap();
        let room = rooms
            .entry(room_name.to_string())
            .or_insert_with(HashSet::new);
        room.insert(client_id.to_string());

        // Set the current room name to the client
        self.current_room_name = Some(room_name.to_owned());
    }

    async fn leave_room(&mut self, client_id: &str, room_name: &str, rooms: &web::Data<Rooms>) {
        // Acquire lock
        let mut rooms = rooms.lock().unwrap();

        // If the room is a valid room!
        if let Some(room) = rooms.get_mut(room_name) {
            // remove the client from the room
            room.remove(client_id);
            self.current_room_name = None;
            if room.is_empty() {
                rooms.remove(room_name);
            }
        }
    }

    async fn broadcast_message(
        &self,
        message: &str,
        rooms: &web::Data<Rooms>,
        clients: &web::Data<Clients>,
    ) {
        // Client must have a room
        assert!(
            self.current_room_name.is_some(),
            "Client tried to cast a message when room was none"
        );

        let rooms = rooms.lock().unwrap();
        let mut clients = clients.lock().unwrap();

        if let Some(room) = rooms.get(&self.current_room_name.clone().unwrap()) {
            for client_id in room {
                if *client_id != self.client_id {
                    if let Some(client) = clients.get_mut(client_id) {
                        let _ = client
                            .get_session()
                            .text(format!(
                                "[{}]: {}",
                                self.user_name.clone().unwrap_or("unknown".to_string()),
                                message
                            ))
                            .await;
                    }
                }
            }
        }
    }
}

async fn ws(
    req: HttpRequest,
    body: web::Payload,
    rooms: web::Data<Rooms>,
    clients: web::Data<Clients>,
) -> actix_web::Result<impl Responder> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    // Generate a unique client ID for this connection
    let client_id = Uuid::new_v4().to_string();

    // Create the new client
    let mut current_client = Client::new(client_id.clone(), session.clone());

    // Add the client to the global list of clients
    clients
        .lock()
        .unwrap()
        .insert(client_id.clone(), current_client.clone());

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
                                current_client.leave_room(&client_id, room, &rooms).await;
                            }

                            // Join the new room
                            current_client
                                .join_room(&client_id, &room_name, &rooms)
                                .await;

                            // Notify the user that it has joined the room
                            session
                                .text(format!("Joined room: {}", room_name))
                                .await
                                .unwrap();
                        } else if text == "/leave" {
                            // Leave the current room
                            if let Some(room) = current_room.take() {
                                current_client.leave_room(&client_id, &room, &rooms).await;
                                session.text("Left the room").await.unwrap();
                            } else {
                                session.text("You are not in any room").await.unwrap();
                            }
                        } else if text.starts_with("/name") {
                            let input: Vec<&str> =
                                text.split_ascii_whitespace().into_iter().collect();
                            if input.len() == 2 {
                                let user_name = input[1];
                                if user_name.is_empty() || user_name.len() < 3 {
                                    // Invalid username
                                    session
                                        .text("User name must be at least 3 chars long")
                                        .await
                                        .unwrap();
                                } else {
                                    // Set the user name
                                    current_client.set_user_name(user_name.to_owned());
                                }
                            } else {
                                // Invalid use of the command
                                session
                                    .text("Set username with /name <user-name>")
                                    .await
                                    .unwrap();
                            }
                        } else {
                            // Broadcast messages to the current room
                            if current_client.has_joined_room() {
                                current_client
                                    .broadcast_message(&text, &rooms, &clients)
                                    .await;
                            } else {
                                session
                                    .text("Join a room first with /join <room_name>")
                                    .await
                                    .unwrap();
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
                current_client.leave_room(&client_id, room, &rooms).await;
            }

            clients.lock().unwrap().remove(&client_id);
        }
    });

    Ok(response)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Creating the rooms and clients
    let rooms: Rooms = Arc::new(Mutex::new(HashMap::new()));
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

    // Server IP and port
    let server_ip = "127.0.0.1";
    let server_port = 8080;

    // Logging
    println!(
        "[INFO] Chat Server running on {}:{}",
        server_ip, server_port
    );

    // Creating and running the HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(rooms.clone()))
            .app_data(web::Data::new(clients.clone()))
            .route("/ws", web::get().to(ws))
    })
    .bind((server_ip, server_port))?
    .run()
    .await
}
