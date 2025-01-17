use actix_files::Files;
use actix_web::{web, App, HttpRequest, HttpServer, Responder};
use actix_ws::Message;
use futures_util::StreamExt as _;
use rustic_reach::{
    core::{
        messages::{ClientMessage, ServerMessage},
        room::room::{ServerRooms, WebRoom},
        user::user::User,
    },
    server::handlers::command_handlers::handle_client_command,
    utils::constants::{ERROR_LOG, INFO_LOG, WARNING_LOG},
};
use std::sync::{Arc, Mutex};

async fn ws(
    req: HttpRequest,
    body: web::Payload,
    rooms: WebRoom,
) -> actix_web::Result<impl Responder> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    // Create the new user
    let mut current_user = User::new(session.clone());

    // Create a new thread for handling the websocket session
    actix_web::rt::spawn({
        async move {
            while let Some(Ok(msg)) = msg_stream.next().await {
                match msg {
                    Message::Text(text) => {
                        let chat_msg: ClientMessage = match serde_json::from_str(&text) {
                            Ok(chat) => chat,
                            Err(_) => {
                                println!("{} Ignored: {}", *WARNING_LOG, text);
                                continue;
                            }
                        };

                        // Handle the message differently based on command or not
                        match chat_msg {
                            // Handle the command
                            ClientMessage::Command(command) => {
                                handle_client_command(&command, &mut current_user, &rooms).await;
                            }

                            // The message is not a command, but a chat message to the room
                            ClientMessage::Chat(chat_message) => {
                                // Log that a chat message has been received
                                println!(
                                    "{} {} wrote a message in {}",
                                    *INFO_LOG, chat_message.sender, chat_message.room
                                );

                                // Acquire lock and broadcast the message for the room
                                if let Ok(server_rooms) = rooms.lock() {
                                    match server_rooms.get_room(&current_user) {
                                        Some(r) => {
                                            let chat_server_message =
                                                ServerMessage::Chat(chat_message);
                                            chat_server_message
                                                .broadcast_msg(r, &current_user)
                                                .await;
                                        }
                                        None => println!(
                                            "{} User was not in a room, could not send message",
                                            *ERROR_LOG
                                        ),
                                    };
                                }
                            }
                        }
                    }

                    // Servers responds to ping messages
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
