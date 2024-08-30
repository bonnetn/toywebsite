use std::net::SocketAddr;
use std::str::FromStr;
use http::{HeaderValue, Response};
use hyper::header::CONTENT_TYPE;
use tokio::net::TcpListener;
use tower::{ServiceBuilder};
use tower_http::compression::{CompressionBody, CompressionLayer};
use crate::app::error::StartupError;
use tower_http::services::fs::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;
use axum::body::{Body, HttpBody};
use tower_http::limit::{RequestBodyLimitLayer};
use crate::app::controller::{Controller};

pub struct Server<C> {
    listen_address: String,
    controller: C,
}

impl<C> Server<C>
    where C: Controller + 'static {
    pub fn new(listen_address: String, controller: C) -> Self {
        Server {
            listen_address,
            controller,
        }
    }

    pub async fn run(&self) -> Result<(), StartupError> {
        let listen_addr = SocketAddr::from_str(&self.listen_address)
            .map_err(|e| StartupError::InvalidListenAddress(e))?;

        let listener = TcpListener::bind(listen_addr)
            .await
            .map_err(|e| StartupError::CouldNotBind(e))?;

        let compression = CompressionLayer::new()
            .gzip(true)
            .deflate(true)
            .br(true)
            .zstd(true);

        let trace = TraceLayer::new_for_http();

        let content_length = SetResponseHeaderLayer::overriding(
            CONTENT_TYPE,
            |response: &Response<CompressionBody<Body>>| {
                if let Some(size) = response.body().size_hint().exact() {
                    Some(HeaderValue::from_str(&size.to_string()).unwrap())
                } else {
                    None
                }
            },
        );

        // let etag = SetResponseHeaderLayer::overriding(
        //     ETAG,
        //     |response: &Response<Body>| {
        //         let body = response.body();
        //         let hash = xxh3_64(body.into_bytes());
        //         let etag = format!("\"{:x}\"", hash);
        //         Some(etag)
        //     },
        // );


        let middlewares = ServiceBuilder::new()
            .layer(trace)
            .layer(RequestBodyLimitLayer::new(1024 * 1024))
            .layer(content_length)
            .layer(compression);

        let router = self.controller.router();
        let app = router
            .nest_service("/static", ServeDir::new("static"))
            .layer(middlewares);


        axum::serve(listener, app)
            .await
            .map_err(|e| StartupError::CouldNotServe(e))
    }
}
