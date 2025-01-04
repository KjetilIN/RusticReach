use actix_files::Files;
use actix_web::{web, App, HttpRequest, HttpServer, Responder};
use actix_ws::Message;
use futures_util::StreamExt as _;
use rustic_reach::{
    core::{
        messages::{ClientMessage, ServerMessage},
        room::room::ServerRooms,
        user::user::User,
    },
    utils::{
        constants::{INFO_LOG, WARNING_LOG},
        hash::hash_str,
        traits::SendServerReply,
    },
};
use std::sync::{Arc, Mutex};

async fn ws(
    req: HttpRequest,
    body: web::Payload,
    rooms: web::Data<Arc<Mutex<ServerRooms>>>,
) -> actix_web::Result<impl Responder> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    // Create the new user
    let mut current_user = User::new(session.clone());

    actix_web::rt::spawn({
        async move {
            while let Some(Ok(msg)) = msg_stream.next().await {
                match msg {
                    Message::Text(text) => {
                        // TODO: deserialize message from server
                        println!("Message: {text}");
                        let chat_msg: ClientMessage = match serde_json::from_str(&text) {
                            Ok(chat) => chat,
                            Err(_) => {
                                println!("{} Ignored: {}", *WARNING_LOG, text);
                                continue;
                            }
                        };

                        // Handle the message differently based on command or not
                        match chat_msg {
                            ClientMessage::Command(command) => {
                                // Info log about message
                                match command {
                                    rustic_reach::core::messages::Command::SetName(new_name) => {
                                        // Changing the name
                                        current_user.set_user_name(new_name);

                                        // Send update message
                                        let msg = ServerMessage::state_update(
                                            &current_user,
                                            "New user name set",
                                        );
                                        msg.send(&mut session).await;
                                    }
                                    rustic_reach::core::messages::Command::JoinRoom(room) => {
                                        // Send success message
                                        let msg = ServerMessage::successful_command("Joined room!");
                                        msg.send(&mut session).await;
                                    }
                                    rustic_reach::core::messages::Command::LeaveRoom => {
                                        // Leave room

                                        // Send update message
                                        let msg =
                                            ServerMessage::state_update(&current_user, "Left room");
                                        msg.send(&mut current_user.get_session()).await;
                                    }
                                    rustic_reach::core::messages::Command::RoomInfo => {
                                        if let Some(room_name) = current_user.get_room_name() {
                                            // Find information about the current room
                                        }
                                    }
                                    rustic_reach::core::messages::Command::AuthUser(user_id) => {
                                        // Set the user id of the user with the given user
                                        current_user.set_id(hash_str(&user_id));

                                        // Send auth message back to user
                                        let msg = ServerMessage::Authenticated;
                                        msg.send(&mut current_user.get_session()).await;
                                    }
                                }
                            }
                            ClientMessage::Chat(chat_message) => {
                                // Log that a chat message has been received
                                println!(
                                    "{} {} wrote a message in {}",
                                    *INFO_LOG, chat_message.sender, chat_message.room
                                );

                                let chat_server_message: ServerMessage =
                                    ServerMessage::Chat(chat_message);

                                // Broadcast this message to the room
                                // TODO: make broadcast message handle closed channels
                                //let _ = current_user.broadcast_message(&chat_server_message, &rooms).await;
                            }
                        }
                    }

                    Message::Ping(bytes) => {
                        if session.pong(&bytes).await.is_err() {
                            break;
                        }
                    }

                    // Maybe here do user cleanup?
                    Message::Close(_) => break,

                    _ => {}
                }
            }
            // TODO: Clean up when the user disconnects
        }
    });

    Ok(response)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // TODO: parse server config file

    // Creating the rooms for the users
    let rooms_mutex = Arc::new(Mutex::new(ServerRooms::with_max_room_count(3)));

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
            .app_data(web::Data::new(rooms_mutex.clone())) // Serving websocket
            .route("/ws", web::get().to(ws))
            // Serving main page
            .service(Files::new("/", "./src/frontend/").index_file("index.html"))
    })
    .bind((server_ip, server_port))?
    .run()
    .await
}
