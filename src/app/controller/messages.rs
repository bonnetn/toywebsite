use askama::Template;
use axum::extract::{Query, State};
use chrono::{DateTime, SecondsFormat, Utc};
use http::StatusCode;
use serde::Deserialize;
use crate::app::controller::{ControllerImpl, EndpointResponse, MyError};
use crate::app::message::repository::Repository;

#[derive(Template)]
#[template(path = "messages.html")]
struct MessagesView<'a> {
    current_page: &'a str,
    entries: Vec<MessageEntry>,
    max_results: usize,
    has_next_page: bool,
    next_page_token: String,
}


struct MessageEntry {
    time: String,
    name: String,
    email: String,
    message: String,
}


#[derive(Debug, Deserialize)]
pub struct ListMessageEntriesQuery {
    max_results: Option<usize>,
    page_token: Option<String>,
}

pub async fn get_messages<R: Repository>(State(c): State<ControllerImpl<R>>, Query(query): Query<ListMessageEntriesQuery>) -> Result<EndpointResponse, MyError> {
    let max_results = query.max_results.unwrap_or(10);

    let page_token = query.page_token
        .map(|s| s.try_into())
        .transpose()
        .map_err(|e| MyError::InvalidField("page_token", e))?;

    let (results, next_page_token) = c.repository.list(max_results, page_token)
        .await
        .map_err(|e| MyError::MessageRepositoryError(e))?;

    let results = results
        .into_iter()
        .map(|msg| {
            let time: DateTime<Utc> = msg.timestamp().into();
            MessageEntry {
                time: time.to_rfc3339_opts(SecondsFormat::Secs, true),
                name: msg.name().to_string(),
                email: msg.email().to_string(),
                message: msg.contents().to_string(),
            }
        })
        .collect();

    let template = MessagesView {
        current_page: "messages",
        entries: results,
        max_results: query.max_results.unwrap_or(10),
        has_next_page: next_page_token.is_some(),
        next_page_token: next_page_token.map(|p| p.to_string()).unwrap_or_default(),
    };

    let body = template
        .render()
        .map_err(MyError::RenderTemplateFailure)?;

    let response = EndpointResponse {
        status: StatusCode::OK,
        content_type: "text/html",
        should_cache: false,
        body,
    };

    Ok(response)
}

