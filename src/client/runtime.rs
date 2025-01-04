use std::{
    process::exit,
    sync::{Arc, Mutex},
};

use actix_codec::Framed;
use actix_web::web::Bytes;
use awc::{ws, BoxedSocket};
use colored::Colorize;
use crossterm::terminal::disable_raw_mode;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt as _, StreamExt,
};
use tokio::{
    select,
    sync::mpsc::{self, UnboundedSender},
    task::LocalSet,
};
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::{
    client::state::ClientState,
    core::messages::{ChatMessage, ClientMessage, Command, ServerMessage},
    utils::{
        constants::{server_message, ERROR_LOG, INFO_LOG, MESSAGE_COMMAND_SYMBOL, SERVER_INFO},
        terminal_ui::TerminalUI,
    },
};

use super::config::ClientConfig;

pub type WsFramedSink = SplitSink<Framed<BoxedSocket, ws::Codec>, ws::Message>;
pub type WsFramedStream = SplitStream<Framed<BoxedSocket, ws::Codec>>;

fn handle_client_stdin(
    input: String,
    terminal_ui: &mut TerminalUI,
    client_state: &mut ClientState,
    message_tx: &mpsc::UnboundedSender<ClientMessage>,
) {
    // Message command only if the command starts with the command symbol
    // This allows users to execute commands when they are messaging
    if input.starts_with(MESSAGE_COMMAND_SYMBOL) {
        // Handle given command
        if let Some(command) = Command::from_str(input.as_str()) {
            match command {
                Command::SetName(new_name) => {
                    // Add message to terminal and change name
                    client_state.user_name = new_name.clone();

                    // Send set name message to server
                    let set_name_message = ClientMessage::Command(Command::SetName(new_name));
                    message_tx.send(set_name_message).unwrap_or_else(|err| {
                        terminal_ui.add_message(format!(
                            "{} Unbounded channel error: {}",
                            *ERROR_LOG, err
                        ));
                    });
                }
                Command::JoinRoom(room_name) => {
                    client_state.room = Some(room_name.clone());

                    // Send join room to server
                    let join_message = ClientMessage::Command(Command::JoinRoom(room_name));
                    message_tx.send(join_message).unwrap_or_else(|err| {
                        terminal_ui.add_message(format!(
                            "{} Unbounded channel error: {}",
                            *ERROR_LOG, err
                        ));
                    });
                }
                Command::LeaveRoom => {
                    // Leaving room
                    client_state.room = None;

                    // Send leave room message
                    let leave_message = ClientMessage::Command(Command::LeaveRoom);
                    message_tx.send(leave_message).unwrap_or_else(|err| {
                        terminal_ui.add_message(format!(
                            "{} Unbounded channel error: {}",
                            *ERROR_LOG, err
                        ));
                    });
                }
                Command::RoomInfo => {
                    if client_state.room.is_some() {
                        // Ask for room info by sending the command
                        let room_info_request = ClientMessage::Command(Command::RoomInfo);
                        message_tx.send(room_info_request).unwrap_or_else(|err| {
                            terminal_ui.add_message(format!(
                                "{} Unbounded channel error: {}",
                                *ERROR_LOG, err
                            ));
                        });
                    } else {
                        // Tell the user to join a room first
                        let room_command_colored =
                            format!("/room").yellow().underline().to_string();
                        terminal_ui.add_message(format!(
                            "{} You need to be in a room before using the {} command",
                            *ERROR_LOG, room_command_colored
                        ));
                    }
                }
                Command::AuthUser(_) => {
                    unimplemented!("Auth from command line")
                }
            }
        } else {
            // Exit command is the client only command
            if input.starts_with("/exit") {
                terminal_ui.add_message(format!("{} Exiting the program...", *INFO_LOG));

                disable_raw_mode().unwrap();
                exit(0);
            } else {
                // Letting the user know what commands they used that was not valid
                let error_command = input.split_ascii_whitespace().collect::<Vec<&str>>()[0]
                    .underline()
                    .red();
                terminal_ui
                    .add_message(format!("{} Unknown command: {}", *ERROR_LOG, error_command));
            }
        }
    } else {
        if client_state.room.is_none() {
            terminal_ui.add_message(format!("{} Please join a room!", *ERROR_LOG));
        } else {
            // The input is text and should be sent to the server
            // TODO: refactor this!!
            let outgoing_msg_result = ChatMessage::create(client_state, input);
            match outgoing_msg_result {
                Ok(message) => {
                    // Send message
                    message_tx
                        .send(ClientMessage::Chat(message.clone()))
                        .unwrap_or_else(|err| {
                            println!("{} Unbounded channel error: {}", *ERROR_LOG, err);
                        });

                    // Print the chat message from the users perspective
                    terminal_ui.add_message(message.format_self());
                }
                Err(_) => {
                    println!(
                        "{} Could not create chat message. Please join a room!",
                        *ERROR_LOG
                    );
                }
            }
        }
    }
}

async fn handle_incoming_messages(
    stream: &mut WsFramedStream,
    sink: &mut WsFramedSink,
    terminal_ui_sender: &UnboundedSender<String>,
    message_rx: &mut UnboundedReceiverStream<ClientMessage>,
) {
    loop {
        select! {
            Some(msg) = stream.next() => match msg {
                Ok(ws::Frame::Text(bytes)) => {
                    match String::from_utf8(bytes.to_vec()) {
                        Ok(valid_str) => {
                            // Parse server message, or ignore the message
                            let server_msg: ServerMessage = match serde_json::from_str(&valid_str) {
                                Ok(msg) => msg,
                                Err(_) => {
                                    continue;
                                }
                            };

                            // Handle message
                            match server_msg {
                                ServerMessage::CommandResult { success: _, message } => {
                                    let server_message = server_message(&message);
                                    terminal_ui_sender.send(server_message).expect("Could not send message over terminal channel")
                                },
                                ServerMessage::StateUpdate { username: _, current_room: _, message } => {
                                    let server_message = server_message(&message);
                                    terminal_ui_sender.send(server_message).expect("Could not send message over terminal channel")
                                },
                                ServerMessage::Chat(chat_message) => {
                                    // Add message to the terminal ui
                                    terminal_ui_sender.send(chat_message.format()).expect("Could not send chat message over terminal channel");
                                },
                                ServerMessage::Authenticated => {
                                    // User is successfully authenticated
                                    let aut_msg = format!("{} Authenticated on the server!", *SERVER_INFO);
                                    terminal_ui_sender.send(aut_msg).expect("Could not send authenticate message over terminal channel");
                                }
                            }
                        },
                        Err(err) => println!("{} Failed to parse text frame: {}", *ERROR_LOG, err),
                    }
                },
                Ok(ws::Frame::Ping(_)) => {
                    sink.send(ws::Message::Pong(Bytes::new())).await.unwrap();
                },
                _ => {}
            },
            Some(message) = message_rx.next() => {
                // Received a chat message from the input thread
                // Message need to be sent to the server
                match serde_json::to_string(&message) {
                    Ok(json) => {
                        // Send the serialized message over the WebSocket
                        sink.send(ws::Message::Text(json.into())).await.unwrap();
                    }
                    Err(err) => {
                        println!("{} Failed to serialize ChatMessage: {}", *ERROR_LOG, err);
                    }
                }
            }
            else => break,
        }
    }
}

pub async fn connect(
    server_ip: String,
    server_port: String,
    client_config: Arc<ClientConfig>,
    terminal_ui: Arc<Mutex<TerminalUI>>,
) {
    let local = LocalSet::new();

    local.spawn_local(async move {
        // Creating another unbounded channel for sending message
        let (client_message_sender, client_message_receiver) = mpsc::unbounded_channel();
        let mut client_message_receiver: UnboundedReceiverStream<ClientMessage> =
            UnboundedReceiverStream::new(client_message_receiver);

        // Create a channel for sending terminal messages to be added to the UI 
        let (terminal_ui_sender, terminal_ui_receiver) = mpsc::unbounded_channel();
        let mut terminal_ui_receiver: UnboundedReceiverStream<String> =
            UnboundedReceiverStream::new(terminal_ui_receiver);

        // Create a terminal ui behind mutex
        if let Ok(mut ui) = terminal_ui.lock() {
            ui.add_message(format!("{} Connecting to {} ...", *INFO_LOG, server_ip));
        }

        // Formatting the websocket connection string
        let ws_url = format!("ws://{server_ip}:{server_port}/ws");

        // Connecting to the given server
        let (_, ws) = awc::Client::new()
            .ws(ws_url)
            .connect()
            .await
            .unwrap_or_else(|err| {
                println!("{} Failed to connect to {}, {}", *ERROR_LOG, server_ip, err);
                exit(1)
            });

        // Successful connection
        if let Ok(mut ui) = terminal_ui.lock() {
            ui.add_message(format!("{} Connected to {}!", *INFO_LOG, server_ip));
        }

        // Creating a sink and stream from the websocket
        // - sink:
        // - stream:
        let (mut sink, mut stream): (WsFramedSink, WsFramedStream) = ws.split();

        // Sent to the server the client token, and confirm 
        //TODO: let server know about the client and its token 

        // Client state to be shared between users
        let mut client_state =
            ClientState::new(client_config.get_token().to_owned(), client_config.get_user_name(&None).to_owned(), None);

        // Creating two threads:
        // - input thread: handle input from the user
        // - message thread: handle incoming messages
        // Spawn asynchronous tasks for handling input and messages
        // Spawn blocking thread for user input
        let input_thread = tokio::spawn(async move {
            loop {
                select! {
                    // Handle messages from the terminal_ui_receiver channel
                    Some(received_message) = terminal_ui_receiver.next() => {
                        println!("MSG from channel: {}", received_message);
                        if let Ok(mut ui) = terminal_ui.lock() {
                            ui.add_message(received_message);
                        }
                    },
                    // Handle direct user input from terminal_ui.handle_input()
                    Ok(input) = async {
                        // Lock the terminal UI and process handle_input()
                        terminal_ui.lock()
                            .map(|mut ui| ui.handle_input())
                            .and_then(|res| Ok(res))
                    } => {
                        if let Ok(Some(input)) = input {
                            if let Ok(mut ui) = terminal_ui.lock() {
                                handle_client_stdin(input, &mut ui, &mut client_state, &client_message_sender);
                            }
                        }
                    }
                }
            }
        });

        // Handle incoming messages on streams and unbounded channel 
        handle_incoming_messages(&mut stream, &mut sink, &terminal_ui_sender, &mut client_message_receiver).await;

        // Wait for the input thread to finish
        if let Err(err) = input_thread.await {
            eprintln!("Input thread panicked: {:?}", err);
        }
    });

    local.await;
}
