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

use crate::shared::{
    constants::{ERROR_LOG, INFO_LOG, MESSAGE_COMMAND_SYMBOL, MESSAGE_LINE_SYMBOL},
    formatted_messages::format_message_string,
};

use super::config::ClientConfig;

type WsFramedSink = SplitSink<Framed<BoxedSocket, ws::Codec>, ws::Message>;
type WsFramedStream = SplitStream<Framed<BoxedSocket, ws::Codec>>;

fn handle_message_commands(
    input: String,
    client_config: &ClientConfig,
    room_name: &mut Option<String>,
    message_tx: &mpsc::UnboundedSender<String>,
) {
    // Message command only if the command starts with the command symbol
    // This allows users to execute commands when they are messaging
    if input.starts_with(MESSAGE_COMMAND_SYMBOL) {
        // Handle given command
        match input.as_str() {
            "/disconnect" => {
                println!("{} Disconnecting...", *INFO_LOG);
            }
            _ => {
                println!("{} Unknown command", *ERROR_LOG);
            }
        }
    } else {
        // The input is text and should be sent to the server
        let user_name = client_config.get_user_name(room_name);
        let message = format_message_string(user_name, (250, 0, 0), &input);

        // Send message
        message_tx.send(message).unwrap_or_else(|err| {
            println!("{} Unbounded channel error: {}", *ERROR_LOG, err);
        });
    }
}

fn handle_user_input(
    cmd_tx: mpsc::UnboundedSender<String>,
    client_config: Arc<ClientConfig>,
    message_tx: &mpsc::UnboundedSender<String>,
) {
    let mut room_name: Option<String> = None;
    loop {
        let mut cmd = String::with_capacity(32);

        print!("{} ", *MESSAGE_LINE_SYMBOL);
        io::stdout().flush().expect("Failed to flush stdout");

        if io::stdin().read_line(&mut cmd).is_err() {
            println!("{} Could not read message input", *ERROR_LOG);
            return;
        }

        handle_message_commands(cmd.clone(), &client_config, &mut room_name, message_tx);
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
                if !cmd.trim().is_empty() {
                    sink.send(ws::Message::Text(cmd.into())).await.unwrap();
                }
            },
            Some(message) = message_rx.next() => {
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
        println!("{} Trying to connect to {} ...", *INFO_LOG, ws_url);

        // Connecting to the given server
        let (_, ws) = awc::Client::new()
            .ws(ws_url)
            .connect()
            .await
            .unwrap_or_else(|err| {
                println!("{} Failed to connect to {}, {}", *ERROR_LOG, server_ip, err);
                exit(1)
            });

        // Creating a sink and stream from the websocket
        // - sink:
        // - stream:
        let (mut sink, mut stream): (WsFramedSink, WsFramedStream) = ws.split();

        // Client Config to be shared between users
        let client_config_clone = Arc::clone(&client_config);

        // Creating two threads:
        // - input thread: handle input from the user
        // - message thread: handle incoming messages
        // Spawn asynchronous tasks for handling input and messages
        // Spawn blocking thread for user input
        let input_thread = thread::spawn(move || {
            handle_user_input(cmd_tx, client_config_clone, &message_tx);
        });

        // Handle incoming WebSocket messages asynchronously in LocalSet
        handle_incoming_messages(&mut stream, &mut sink, &mut cmd_rx, &mut message_rx).await;

        // Wait for the input thread to finish
        input_thread.join().expect("Input thread panicked");
    });

    local.await;
}
