use std::{net::{TcpListener}, process::exit, error::Error};
use std::result::Result;
use std::io::Write;

fn main() -> Result<(), Box<dyn Error>>{

    // Address to connect to
    let address = "127.0.0.1:8080";

    // TCP Listener 
    let tcp_listener = TcpListener::bind(&address).map_err(|err|{
        eprint!("ERROR: Error during binding a TCP listener : {err}");
        return err;
    })?;

    // Logging info 
    println!("INFO: Listening to {address}...");

    // Listening to streams
    for  stream in tcp_listener.incoming(){
        match  stream {
            Ok(mut stream) => {
                // Read the stream
                let _ = writeln!(stream, "Hello world").map_err(|err| {
                    eprint!("ERROR: could not handle stream write {err}");
                });
            },
            Err(err) => eprintln!("ERROR: Could not read from stream : {err}"),
        }
    }

    

    Ok(())
}
