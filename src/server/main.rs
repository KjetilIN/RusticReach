use actix_files::Files;
use actix_web::{web, App, HttpRequest, HttpServer, Responder};
use actix_ws::{Message, Session};
use futures_util::StreamExt as _;
use rustic_reach::{
    server::{
        room::Rooms,
        user::{User, Users},
    },
    shared::{
        cmd::{command::CommandType, message_commands::MESSAGE_COMMANDS},
        constants::ERROR_LOG,
    },
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

type WebRoom = web::Data<Arc<Mutex<HashMap<String, std::collections::HashSet<String>>>>>;

async fn handle_join(
    session: &mut Session,
    text: String,
    current_room: &mut Option<String>,
    user: &mut User,
    user_id: &String,
    rooms: &WebRoom,
) {
    let room_name = text.strip_prefix("/join ").unwrap().to_string();

    // Leave the current room if necessary
    if let Some(room) = &current_room {
        user.leave_room(&user_id, room, &rooms).await;
    }

    // Log join message
    //TODO: FIX this
    println!(
        "[INFO] User {} is joining in room: {}",
        <Option<String> as Clone>::clone(&user.get_user_name()).unwrap(),
        room_name
    );

    // Join the new room
    user.join_room(&user_id, &room_name, &rooms).await;

    // Notify the user that it has joined the room
    session
        .text(format!("Joined room: {}", room_name))
        .await
        .unwrap();
}

async fn handle_leave(
    session: &mut Session,
    current_room: &mut Option<String>,
    user: &mut User,
    user_id: &String,
    rooms: &WebRoom,
) {
    // Leave the current room
    if let Some(room) = current_room.take() {
        user.leave_room(&user_id, &room, &rooms).await;
        session.text("Left the room").await.unwrap();
    } else {
        session.text("You are not in any room").await.unwrap();
    }
}

async fn handle_name(session: &mut Session, text: String, user: &mut User) {
    let input: Vec<&str> = text.split_ascii_whitespace().into_iter().collect();
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
            user.set_user_name(user_name.to_owned());
        }
    } else {
        // Invalid use of the command
        session
            .text("Set username with /name <user-name>")
            .await
            .unwrap();
    }
}

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
            let mut current_room: Option<String> = None;

            while let Some(Ok(msg)) = msg_stream.next().await {
                match msg {
                    Message::Text(text) => {
                        let text = text.trim();
                        println!("Message: {}", text);

                        // Check if given message is an command
                        if let Some(command) = MESSAGE_COMMANDS.retrieve_command(text.to_owned()) {
                            if let Some(command_type) = command.get_type() {
                                match command_type {
                                    CommandType::Join => {
                                        handle_join(
                                            &mut session,
                                            text.to_owned(),
                                            &mut current_room,
                                            &mut current_user,
                                            &user_id,
                                            &rooms,
                                        )
                                        .await
                                    }
                                    CommandType::Leave => {
                                        handle_leave(
                                            &mut session,
                                            &mut current_room,
                                            &mut current_user,
                                            &user_id,
                                            &rooms,
                                        )
                                        .await
                                    }
                                    CommandType::Name => handle_name(&mut session, text.to_owned(), &mut current_user).await, 
                                }
                            } else {
                                // Command detected, should not happen
                                println!("{} Message command not included in list of valid commands: {text}", *ERROR_LOG);
                            }
                        } else {
                            // Message was not a command: broadcast it!
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
