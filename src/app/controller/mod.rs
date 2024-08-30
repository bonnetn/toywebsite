mod home;
mod about;
mod not_found;
mod contact;
mod messages;

use std::fmt::Display;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Router;
use axum::routing::{MethodRouter};
use http::{header, HeaderMap};
use crate::app::message::repository::Repository;
use crate::app::validation;


pub trait Controller: Clone + Send + Sync  {
    fn router(&self) -> Router;
}

#[derive(Debug, Clone)]
pub struct ControllerImpl<R> {
    repository: R,
}

impl<R> ControllerImpl<R> {
    pub fn new(repository: R) -> ControllerImpl<R> {
        ControllerImpl { repository }
    }
}

impl<R> Controller for ControllerImpl<R>
    where R: Repository + 'static {
    fn router(&self) -> Router {
        let home = MethodRouter::new()
            .get(home::get_home::<Self>);

        let about = MethodRouter::new()
            .get(about::get_about);

        let contact = MethodRouter::new()
            .get(contact::get_contact)
            .post(contact::post_contact::<R>);

        let messages = MethodRouter::new()
            .get(messages::get_messages::<R>);

        let not_found = MethodRouter::new()
            .get(not_found::not_found);

        Router::new()
            .route("/", home)
            .route("/about", about)
            .route("/contact", contact)
            .route("/messages", messages)
            .fallback(not_found)
            .with_state(self.clone())
    }
}

struct EndpointResponse {
    status: StatusCode,
    content_type: &'static str,
    should_cache: bool,
    body: String,
}

impl IntoResponse for EndpointResponse {
    fn into_response(self) -> axum::response::Response {
        let mut headers = HeaderMap::new();
        if !self.should_cache {
            headers.insert(header::CACHE_CONTROL, "no-cache".parse().unwrap());
        } else {
            headers.insert(header::CACHE_CONTROL, "public, max-age=604800".parse().unwrap());
        }

        headers.insert(header::CONTENT_TYPE, self.content_type.parse().unwrap());

        (self.status, headers, self.body).into_response()
    }
}


#[derive(Debug)]
enum MyError {
    InvalidField(&'static str, validation::Error),
    RenderTemplateFailure(askama::Error),
    MessageRepositoryError(Box<dyn std::error::Error + Send + Sync >),
}

impl Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MyError::RenderTemplateFailure(e) => {
                write!(f, "failed to render template: {}", e)
            }
            MyError::InvalidField(name, e) =>
                write!(f, "invalid field {}: {}", name, e),
            MyError::MessageRepositoryError(e) =>
                write!(f, "message repository: {}", e),
        }
    }
}

impl std::error::Error for MyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MyError::RenderTemplateFailure(e) => Some(e),
            MyError::InvalidField(_, e) => Some(e),
            MyError::MessageRepositoryError(e) => Some(e.as_ref()),
        }
    }
}

impl IntoResponse for MyError {
    fn into_response(self) -> axum::response::Response {
        let (status, body) = match self {
            MyError::RenderTemplateFailure(_) =>
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error".to_string()),
            MyError::InvalidField(_, _) =>
                (StatusCode::BAD_REQUEST, self.to_string()),
            MyError::MessageRepositoryError(_) =>
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error".to_string()),
        };

        (status, body).into_response()
    }
}







