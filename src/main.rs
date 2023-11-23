// Import Mio
extern crate mio;
use mio::{Poll, Events};
use std::process::exit;

// Represents a websocket server 
struct WebSocketServer;


fn main() {
    // Hold all events for a poll
    let mut events = Events::with_capacity(1024);

    // Allows a program to monitor a large number of requests 
    let mut poll =  match Poll::new(){
        Ok(pull) => pull,
        Err(e) => {
            eprint!("ERROR: creating poll: {e}");
            exit(1);
        },
    };
}
