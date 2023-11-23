use std::{net::{TcpListener}, error::Error, fmt::Display};
use std::result::Result;
use std::io::Write;


// Boolean for censoring information
const IS_CENSORED: bool = false;


// Struct that represents the Censored information
// Inner is the part that will be displayed, of a generic type
struct Censor<T>{
    inner: T
}

impl<T> Censor<T>{
    fn new(inner: T)-> Self{
        Self{inner}
    }

}

// Implementation for displaying the inner 
impl <T: std::fmt::Display> Display for Censor<T>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if IS_CENSORED {
            let _ = writeln!(f, "[CENSORED]").map_err(|err|{
                eprint!("ERROR: censor did not work on write : {err}", err = Censor::new(err));
            });
        }else{
            let _ = writeln!(f, "{inner}", inner = &self.inner).map_err(|err|{
                eprintln!("ERROR: writing inner had an error : {err}", err = Censor::new(err));
            });
        }

        Ok(())
    }
    
}


fn client(){
    todo!();
    // 35 minutes in
}



fn main() -> Result<(), Box<dyn Error>>{

    // Address to connect to
    let address = "0.0.0.0:8080";

    // TCP Listener 
    let tcp_listener = TcpListener::bind(&address).map_err(|err|{
        eprint!("ERROR: Error during binding a TCP listener : {err}", err = Censor::new(&err));
        return err;
    })?;

    // Logging info, and maybe censor it 
    println!("INFO: Listening to {address}...", address = Censor::new(address));

    // Listening to streams
    for  stream in tcp_listener.incoming(){
        match  stream {
            Ok(mut stream) => {
                // Read the stream
                let _ = writeln!(stream, "Hello world").map_err(|err| {
                    eprint!("ERROR: could not handle stream write {err}", err = Censor::new(err));
                });
            },
            Err(err) => eprintln!("ERROR: Could not read from stream : {err}" , err = Censor::new(err)),
        }
    }

    

    Ok(())
}
