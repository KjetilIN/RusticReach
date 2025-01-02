use std::{
    io::{self, Write},
    process::exit,
    sync::Arc,
    thread,
};

use actix_codec::Framed;
use actix_web::web::Bytes;
use awc::{ws, BoxedSocket};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt as _, StreamExt,
};
use tokio::{select, sync::mpsc, task::LocalSet};
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::{
    client::state::ClientState,
    core::messages::{ChatMessage, ClientMessage, Command},
    utils::{
        constants::{ERROR_LOG, INFO_LOG, MESSAGE_COMMAND_SYMBOL, MESSAGE_LINE_SYMBOL},
        terminal_ui::{self, TerminalUI},
        traits::SendServerReply,
    },
};

use super::config::ClientConfig;

type WsFramedSink = SplitSink<Framed<BoxedSocket, ws::Codec>, ws::Message>;
type WsFramedStream = SplitStream<Framed<BoxedSocket, ws::Codec>>;

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
                    terminal_ui.add_message(format!(
                        "{} Client state change name to {}",
                        *INFO_LOG, new_name
                    ));
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
                    terminal_ui.add_message(format!("{} Change room to {}", *INFO_LOG, room_name));
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
                    terminal_ui.add_message(format!("{} Client state, user left room", *INFO_LOG));
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
            }
        } else {
            terminal_ui.add_message(format!("{} Unknown command", *ERROR_LOG));
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
    message_rx: &mut UnboundedReceiverStream<ClientMessage>,
) {
    loop {
        select! {
            Some(msg) = stream.next() => match msg {
                Ok(ws::Frame::Text(frame)) => {
                    match String::from_utf8(frame.to_vec()) {
                        Ok(valid_str) => {
                            // TODO: handle incoming server message
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

pub async fn connect(server_ip: String, server_port: String, client_config: Arc<ClientConfig>) {
    let local = LocalSet::new();

    local.spawn_local(async move {
        // Creating another unbounded channel for sending message
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        let mut message_rx: UnboundedReceiverStream<ClientMessage> =
            UnboundedReceiverStream::new(message_rx);

        // Initialize terminal UI
        enable_raw_mode().unwrap();
        let mut terminal_ui = TerminalUI::new().unwrap();

        // Formatting the websocket connection string
        let ws_url = format!("ws://{server_ip}:{server_port}/ws");
        terminal_ui.add_message(format!("[INFO] Connecting to {} ...", server_ip));

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
        terminal_ui.add_message(format!("[INFO] Connected to {}!", server_ip));

        // Creating a sink and stream from the websocket
        // - sink:
        // - stream:
        let (mut sink, mut stream): (WsFramedSink, WsFramedStream) = ws.split();

        // Client state to be shared between users
        let mut client_state =
            ClientState::new(client_config.get_user_name(&None).to_owned(), None);

        // Creating two threads:
        // - input thread: handle input from the user
        // - message thread: handle incoming messages
        // Spawn asynchronous tasks for handling input and messages
        // Spawn blocking thread for user input
        let input_thread = thread::spawn(move || loop {
            if let Ok(Some(input)) = terminal_ui.handle_input() {
                handle_client_stdin(input, &mut terminal_ui, &mut client_state, &message_tx);
            }
        });

        // Handle incoming WebSocket messages asynchronously in LocalSet
        handle_incoming_messages(&mut stream, &mut sink, &mut message_rx).await;

        // Wait for the input thread to finish
        input_thread.join().expect("Input thread panicked");
        disable_raw_mode().unwrap();
    });

    local.await;
}
