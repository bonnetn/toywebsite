use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use tokio::net::TcpListener;
use crate::app::error::StartupError;
use crate::app::handler::Handler;

pub struct Server {
    listen_address: String,
    handler: Arc<Handler>,
}

impl Server {
    pub fn new(listen_address: String, handler: Arc<Handler>) -> Self {
        Server{
            listen_address,
            handler,
        }
    }

    pub async fn run(&self) -> Result<(), StartupError> {
        let listen_addr = SocketAddr::from_str(&self.listen_address)
            .map_err(|e| StartupError::InvalidListenAddress(e))?;

        let listener = TcpListener::bind(listen_addr)
            .await
            .map_err(|e| StartupError::CouldNotBind(e))?;

        loop {
            let handler = Arc::clone(&self.handler);

            let (stream, _) = listener.accept()
                .await
                .map_err(|e| StartupError::CouldNotAcceptConnection(e))?;

            tokio::spawn(async move {
                let serve = service_fn(move |req| {
                    let handler = Arc::clone(&handler);

                    async move {
                        handler.handle(req).await
                    }
                });

                let io = TokioIo::new(stream);
                auto::Builder::new(TokioExecutor::new())
                    .serve_connection(io, serve)
                    .await
                    .map_err(|e| eprintln!("Error serving connection: {:?}", e))
                    .ok();
            });
        }
    }
}