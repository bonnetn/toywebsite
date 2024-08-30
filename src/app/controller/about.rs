use askama::Template;
use http::StatusCode;
use crate::app::controller::{EndpointResponse, MyError};

#[derive(Template)]
#[template(path = "about.html")]
struct AboutView<'a> {
    current_page: &'a str,
}

pub async fn get_about() -> Result<EndpointResponse, MyError> {
    let template = AboutView { current_page: "about" };
    let body = template.render().map_err(MyError::RenderTemplateFailure)?;
    let response = EndpointResponse {
        status: StatusCode::OK,
        content_type: "text/html",
        should_cache: true,
        body,
    };

    Ok(response)
}
