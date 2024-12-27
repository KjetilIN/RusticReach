use std::io;
use actix_web::{middleware::Logger, web, App, HttpRequest, HttpServer, Responder};
use actix_ws::Message;
use futures_util::StreamExt as _;

async fn ws(req: HttpRequest, body: web::Payload) -> actix_web::Result<impl Responder> {
    // Handle an async request 
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Ping(bytes) => {
                    if session.pong(&bytes).await.is_err() {
                        return;
                    }
                }

                Message::Text(msg) => println!("Got text: {msg}"),
                _ => break,
            }
        }

        let _ = session.close(None).await;
    });

    Ok(response)
}

#[tokio::main]
async fn main() -> io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .route("/ws", web::get().to(ws))
            .wrap(Logger::default())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}