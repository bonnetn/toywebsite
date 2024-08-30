use askama::Template;
use crate::app::controller::{EndpointResponse, MyError};

#[derive(Template)]
#[template(path = "not_found.html")]
struct NotFoundView<'a> {
    current_page: &'a str,
}

pub async fn not_found() -> Result<EndpointResponse, MyError> {
    let template = NotFoundView { current_page: "not_found" };
    let body = template.render().map_err(MyError::RenderTemplateFailure)?;
    let response = EndpointResponse {
        status: http::StatusCode::NOT_FOUND,
        content_type: "text/html",
        should_cache: true,
        body,
    };

    Ok(response)
}
