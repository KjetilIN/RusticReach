use actix_files::Files;
use actix_web::{web, App, HttpRequest, HttpServer, Responder};
use actix_ws::{Message, Session};
use futures_util::StreamExt as _;
use rustic_reach::server::{room::Rooms, user::{User, Users}};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};
use uuid::Uuid;

async fn ws(
    req: HttpRequest,
    body: web::Payload,
    rooms: web::Data<Rooms>,
    users: web::Data<Users>,
) -> actix_web::Result<impl Responder> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    // Generate a unique user ID for this connection
    let user_id = Uuid::new_v4().to_string();

    // Create the new user
    let mut current_user = User::new(user_id.clone(), session.clone());

    // Add the user to the global list of users
    users
        .lock()
        .unwrap()
        .insert(user_id.clone(), current_user.clone());

    actix_web::rt::spawn({
        let rooms = rooms.clone();
        let users = users.clone();

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
                                current_user.leave_room(&user_id, room, &rooms).await;
                            }

                            // Log join message
                            println!(
                                "[INFO] User {} is joining in room: {}",
                                current_user.get_user_name(),
                                room_name
                            );

                            // Join the new room
                            current_user.join_room(&user_id, &room_name, &rooms).await;

                            // Notify the user that it has joined the room
                            session
                                .text(format!("Joined room: {}", room_name))
                                .await
                                .unwrap();
                        } else if text == "/leave" {
                            // Leave the current room
                            if let Some(room) = current_room.take() {
                                current_user.leave_room(&user_id, &room, &rooms).await;
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
                                    current_user.set_user_name(user_name.to_owned());
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
                            if current_user.has_joined_room() {
                                current_user.broadcast_message(&text, &rooms, &users).await;
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

            // Clean up when the user disconnects
            if let Some(room) = &current_room {
                current_user.leave_room(&user_id, room, &rooms).await;
            }

            users.lock().unwrap().remove(&user_id);
        }
    });

    Ok(response)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Creating the rooms and users
    let rooms: Rooms = Arc::new(Mutex::new(HashMap::new()));
    let users: Users = Arc::new(Mutex::new(HashMap::new()));

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
            .app_data(web::Data::new(users.clone()))
            // Serving websocket
            .route("/ws", web::get().to(ws))
            // Serving main page
            .service(Files::new("/", "./src/frontend/").index_file("index.html"))
    })
    .bind((server_ip, server_port))?
    .run()
    .await
}
