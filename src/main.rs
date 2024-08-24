use std::sync::Arc;
use crate::app::error::StartupError;
use crate::app::handler::Handler;
use crate::app::message::Repository;
use crate::app::server::Server;

mod app;

#[tokio::main]
async fn main() {
    match run().await {
        Ok(_) => println!("Server finished"),
        Err(e) => eprintln!("Error: {:?}", e),
    }
}

async fn run() -> Result<(), StartupError> {
    let filename = "messages.json";
    let repository = Arc::new(Repository::new(filename));
    let handler = Handler::new(repository).await?;
    let server = Server::new(
        "127.0.0.1:3000".to_string(),
        Arc::new(handler),
    );

    server.run().await
}
