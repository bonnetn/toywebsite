use std::sync::Arc;
use sqlx::sqlite::SqlitePoolOptions;
use crate::app::error::StartupError;
use crate::app::handler::Handler;
use crate::app::message::repository::Repository;
use crate::app::message::repository::sqlite::SQLiteRepository;
use crate::app::server::Server;

mod app;

#[tokio::main]
async fn main() {
    match run().await {
        Ok(_) => println!("Server finished"),
        Err(e) => eprintln!("error: {}", e),
    }
}

async fn run() -> Result<(), StartupError> {
    let conn = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("db/database.sqlite")
        .await
        .map_err(|e| StartupError::CannotCreateConnectionPool(e))?;

    let sqlite_repository = SQLiteRepository::new(conn.clone());
    let sqlite_repository: Arc<dyn Repository> = Arc::new(sqlite_repository);

    let handler = Handler::new(sqlite_repository).await?;
    let server = Server::new(
        "127.0.0.1:3000".to_string(),
        Arc::new(handler),
    );

    server.run().await
}
