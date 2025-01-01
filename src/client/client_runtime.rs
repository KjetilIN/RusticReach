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
    shared::{
        cmd::command::CommandType,
        constants::{ERROR_LOG, INFO_LOG, MESSAGE_COMMAND_SYMBOL, MESSAGE_LINE_SYMBOL},
        formatted_messages::format_message_string,
    },
};

use super::config::ClientConfig;

type WsFramedSink = SplitSink<Framed<BoxedSocket, ws::Codec>, ws::Message>;
type WsFramedStream = SplitStream<Framed<BoxedSocket, ws::Codec>>;

fn handle_message_commands(
    input: String,
    client_state: &mut ClientState,
    message_tx: &mpsc::UnboundedSender<String>,
) {
    // Message command only if the command starts with the command symbol
    // This allows users to execute commands when they are messaging
    if input.starts_with(MESSAGE_COMMAND_SYMBOL) {
        // Handle given command
        if let Some(command_type) = CommandType::from_str(input.as_str()) {
            match command_type {
                CommandType::Join => {
                    let input_parts: Vec<&str> = input.split_ascii_whitespace().collect();
                    if input_parts.len() == 2 {
                        println!("Joining room: {}", client_state.user_name);
                        client_state.room = Some(input_parts[1].to_owned());
                    }
                }
                CommandType::Leave => {
                    println!("Leave server");
                    client_state.room = None;
                }
                CommandType::Name => {
                    println!("Command name");
                    let input_parts: Vec<&str> = input.split_ascii_whitespace().collect();
                    println!("{:?}", input_parts);
                    if input_parts.len() == 2 {
                        println!("Rename: Client state username: {}", client_state.user_name);
                        client_state.user_name = input_parts[1].to_owned();
                    }
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
            let user_name = &client_state.user_name;
            let out_going_message = format_message_string(&user_name, (250, 0, 0), &input);

            // Send message
            message_tx.send(out_going_message).unwrap_or_else(|err| {
                println!("{} Unbounded channel error: {}", *ERROR_LOG, err);
            });
        }
    }
}

fn handle_user_input(
    cmd_tx: mpsc::UnboundedSender<String>,
    client_state: &mut ClientState,
    message_tx: &mpsc::UnboundedSender<String>,
) {
    loop {
        let mut cmd = String::with_capacity(32);

        print!("{} ", *MESSAGE_LINE_SYMBOL);
        io::stdout().flush().expect("Failed to flush stdout");

        if io::stdin().read_line(&mut cmd).is_err() {
            println!("{} Could not read message input", *ERROR_LOG);
            return;
        }

        handle_message_commands(cmd.clone(), client_state, message_tx);
        if let Err(err) = cmd_tx.send(cmd) {
            println!("{} Failed to send command: {}", *ERROR_LOG, err);
            exit(1);
        }
    }
}

async fn handle_incoming_messages(
    stream: &mut WsFramedStream,
    sink: &mut WsFramedSink,
    cmd_rx: &mut UnboundedReceiverStream<String>,
    message_rx: &mut UnboundedReceiverStream<String>,
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
            Some(cmd) = cmd_rx.next() => {
                println!("message stream");
                if !cmd.trim().is_empty() {
                    sink.send(ws::Message::Text(cmd.into())).await.unwrap();
                }
            },
            Some(message) = message_rx.next() => {
                println!("message stream");
                if !message.trim().is_empty() {
                    sink.send(ws::Message::Text(message.into())).await.unwrap();
                }
            }
            else => break,
        }
    }
}

pub async fn connect(server_ip: String, server_port: String, client_config: Arc<ClientConfig>) {
    let local = LocalSet::new();

    local.spawn_local(async move {
        // Creating an unbounded channel that allows us to;
        // - Send terminal commands to and from the message thread
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        let mut cmd_rx: UnboundedReceiverStream<String> = UnboundedReceiverStream::new(cmd_rx);

        // Creating another unbounded channel for sending message
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        let mut message_rx: UnboundedReceiverStream<String> =
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
            handle_user_input(cmd_tx, &mut client_state, &message_tx);
        });

        // Handle incoming WebSocket messages asynchronously in LocalSet
        handle_incoming_messages(&mut stream, &mut sink, &mut cmd_rx, &mut message_rx).await;

        // Wait for the input thread to finish
        input_thread.join().expect("Input thread panicked");
    });

    local.await;
}
