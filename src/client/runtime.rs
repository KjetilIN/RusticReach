use std::{
    io::{self, Write},
    process::exit,
    sync::Arc,
    thread,
};

use actix_codec::Framed;
use actix_web::web::Bytes;
use awc::{ws, BoxedSocket};
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
        traits::SendServerReply,
    },
};

use super::config::ClientConfig;

type WsFramedSink = SplitSink<Framed<BoxedSocket, ws::Codec>, ws::Message>;
type WsFramedStream = SplitStream<Framed<BoxedSocket, ws::Codec>>;

fn handle_client_stdin(
    input: String,
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
                    println!("{} Client state change name to {}", *INFO_LOG, new_name);
                    client_state.user_name = new_name.clone();

                    // Send set name message to server
                    let set_name_message = ClientMessage::Command(Command::SetName(new_name));
                    message_tx.send(set_name_message).unwrap_or_else(|err| {
                        println!("{} Unbounded channel error: {}", *ERROR_LOG, err);
                    });
                }
                Command::JoinRoom(room_name) => {
                    println!("{} Change room to {}", *INFO_LOG, room_name);
                    client_state.room = Some(room_name.clone());

                    // Send join room to server
                    let join_message = ClientMessage::Command(Command::JoinRoom(room_name));
                    message_tx.send(join_message).unwrap_or_else(|err| {
                        println!("{} Unbounded channel error: {}", *ERROR_LOG, err);
                    });
                }
                Command::LeaveRoom => {
                    println!("{} Client state, user left room", *INFO_LOG);
                    client_state.room = None;

                    // Send leave room message
                    let leave_message = ClientMessage::Command(Command::LeaveRoom);
                    message_tx.send(leave_message).unwrap_or_else(|err| {
                        println!("{} Unbounded channel error: {}", *ERROR_LOG, err);
                    });
                }
            }
        } else {
            println!("{} Unknown command", *ERROR_LOG)
        }
    } else {
        if client_state.room.is_none() {
            println!("{} Please join a room!", *ERROR_LOG);
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
                    println!("{}", message.format_self());
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

fn handle_user_input(
    client_state: &mut ClientState,
    message_tx: &mpsc::UnboundedSender<ClientMessage>,
) {
    loop {
        let mut cmd = String::with_capacity(32);

        print!("{} ", *MESSAGE_LINE_SYMBOL);
        io::stdout().flush().expect("Failed to flush stdout");

        if io::stdin().read_line(&mut cmd).is_err() {
            println!("{} Could not read message input", *ERROR_LOG);
            return;
        }

        handle_client_stdin(cmd.clone(), client_state, message_tx);
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
                        Ok(valid_str) => println!("{}", valid_str),
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

        // Formatting the websocket connection string
        let ws_url = format!("ws://{server_ip}:{server_port}/ws");
        println!("{} Connecting to {} ...", *INFO_LOG, server_ip);

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
        println!("{} Connected to {}!", *INFO_LOG, server_ip);

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
        let input_thread = thread::spawn(move || {
            handle_user_input(&mut client_state, &message_tx);
        });

        // Handle incoming WebSocket messages asynchronously in LocalSet
        handle_incoming_messages(&mut stream, &mut sink, &mut message_rx).await;

        // Wait for the input thread to finish
        input_thread.join().expect("Input thread panicked");
    });

    local.await;
}
