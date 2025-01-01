use actix_files::Files;
use actix_web::{web, App, HttpRequest, HttpServer, Responder};
use actix_ws::Message;
use futures_util::StreamExt as _;
use rustic_reach::{
    core::{messages::{ChatMessage, ClientMessage}, user::{User, Users}},
    server::{
        handlers::ws_handlers::{handle_join, handle_leave, handle_name},
        room::Rooms,
    },
    utils::constants::{ERROR_LOG, INFO_LOG, WARNING_LOG},
};
use std::{
    collections::HashMap,
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
        let rooms: web::Data<Arc<Mutex<HashMap<String, std::collections::HashSet<String>>>>> =
            rooms.clone();
        let users = users.clone();

        async move {
            while let Some(Ok(msg)) = msg_stream.next().await {
                match msg {
                    Message::Text(text) => {
                        // TODO: deserialize message from server
                        println!("Message: {text}");
                        let chat_msg: ClientMessage = match serde_json::from_str(&text){
                            Ok(chat) => chat,
                            Err(_) => {
                                println!("{} Ignored: {}", *WARNING_LOG, text);
                                continue;
                            },
                        };

                        // Handle the message differently based on command or not
                        match chat_msg{
                            ClientMessage::Command(command) => {
                                // Info log about message
                                match command {
                                    rustic_reach::core::messages::Command::SetName(new_name) => {
                                        // Changing the name
                                        current_user.set_user_name(new_name);

                                        // TODO: respond
                                    },
                                    rustic_reach::core::messages::Command::JoinRoom(room) => {
                                        // TODO: reconnect to another server logic
                                        println!("{} Reconnecting to another server has not been implemented", *WARNING_LOG);
                                    },
                                    rustic_reach::core::messages::Command::LeaveRoom => {
                                        current_user.leave_room(&user_id, &rooms).await;
                                    },
                                }
                            },
                            ClientMessage::Chat(chat_message) => {
                                // Log that a chat message has been received 
                                println!("{} {} wrote a message in {}", *INFO_LOG, chat_message.sender, chat_message.room);

                                // Broadcast this message to the room
                                // TODO: make broadcast message handle closed channels
                                let _ = current_user.broadcast_message(&chat_message, &rooms, &users).await;
                            },
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
            if let Some(_) = &current_user.take_room() {
                current_user.leave_room(&user_id, &rooms).await;
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
