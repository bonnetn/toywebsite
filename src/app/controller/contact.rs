use std::time::SystemTime;
use askama::Template;
use axum::extract::State;
use axum::Form;
use http::StatusCode;
use serde::Deserialize;
use crate::app::controller::{ControllerImpl, EndpointResponse, MyError};
use crate::app::message::Message;
use crate::app::message::repository::Repository;

#[derive(Template)]
#[template(path = "contact.html")]
struct ContactView<'a> {
    current_page: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct ContactFormData {
    name: String,
    email: String,
    message: String,
}

pub async fn get_contact() -> Result<EndpointResponse, MyError> {
    let template = ContactView { current_page: "contact" };
    let body = template.render().map_err(MyError::RenderTemplateFailure)?;
    let response = EndpointResponse {
        status: StatusCode::OK,
        content_type: "text/html",
        should_cache: true,
        body,
    };

    Ok(response)
}

pub async fn post_contact<R: Repository>(State(c): State<ControllerImpl<R>>, Form(form_data): Form<ContactFormData>) -> Result<EndpointResponse, MyError> {
    let timestamp = SystemTime::now();
    let name = form_data.name.try_into()
        .map_err(|e| MyError::InvalidField("name", e))?;

    let email = form_data.email.try_into()
        .map_err(|e| MyError::InvalidField("email", e))?;

    let contents = form_data.message.try_into()
        .map_err(|e| MyError::InvalidField("message", e))?;

    let message = Message::new(timestamp, name, email, contents);

    c.repository.create(&message)
        .await
        .map_err(|e| MyError::MessageRepositoryError(e))?;

    let body = "<p>Thank you for your message!</p>".to_string();
    let response = EndpointResponse {
        status: StatusCode::OK,
        content_type: "text/html",
        should_cache: false,
        body,
    };

    Ok(response)
}
