use askama::Template;
use axum::extract::State;
use http::{StatusCode};
use crate::app::controller::{Controller, EndpointResponse, MyError};

#[derive(Template)]
#[template(path = "home.html")]
struct HomeView<'a> {
    current_page: &'a str,
}


pub async fn get_home<C: Controller>(State(_): State<C>) -> Result<EndpointResponse, MyError> {
    let template = HomeView { current_page: "home" };
    let body = template.render().map_err(MyError::RenderTemplateFailure)?;
    let response = EndpointResponse {
        status: StatusCode::OK,
        content_type: "text/html",
        should_cache: true,
        body,
    };

    Ok(response)
}
