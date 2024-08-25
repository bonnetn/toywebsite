use std::collections::HashMap;
use std::convert::Infallible;
use std::io::{Write};
use std::sync::Arc;
use std::time::SystemTime;
use askama::Template;
use chrono::{DateTime, SecondsFormat, Utc};
use flate2::write::GzEncoder;
use futures::future::join_all;
use http_body_util::{BodyExt, Full};
use hyper::body::{Body, Bytes, Incoming};
use hyper::{Request, Response};
use serde::Deserialize;
use tokio::fs;
use xxhash_rust::const_xxh3::xxh3_64;
use crate::app::error::{HTTPError, StartupError};
use crate::app::message::{Message};
use crate::app::message::repository::Repository;


#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate<'a> {
    current_page: &'a str,
}

#[derive(Template)]
#[template(path = "contact.html")]
struct ContactTemplate<'a> {
    current_page: &'a str,
}

#[derive(Template)]
#[template(path = "about.html")]
struct AboutTemplate<'a> {
    current_page: &'a str,
}

#[derive(Template)]
#[template(path = "messages.html")]
struct MessagesTemplate<'a> {
    current_page: &'a str,
}

#[derive(Template)]
#[template(path = "message_entries.html")]
struct MessageEntriesTemplate {
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
struct ContactFormData {
    name: String,
    email: String,
    message: String,
}

#[derive(Debug, Deserialize)]
struct ListMessageEntriesQuery {
    max_results: Option<usize>,
    page_token: Option<String>,
}

pub struct Handler {
    repository: Arc<dyn Repository>,
    static_resources: HashMap<String, StaticResource>,
}

impl Handler {
    pub async fn new(repository: Arc<dyn Repository>) -> Result<Self, StartupError> {
        let paths = [
            ("/styles.css", "static/styles.css", "text/css"),
            ("/favicon.ico", "static/favicon.ico", "image/x-icon"),
        ];

        let futures = paths
            .iter()
            .map(|(route, file_path, content_type)| {
                StaticResource::load(route, file_path, content_type)
            });

        let resources = join_all(futures)
            .await
            .into_iter()
            .collect::<Result<Vec<StaticResource>, StartupError>>()?;

        let static_resources = resources
            .into_iter()
            .map(|resource| (resource.route.clone(), resource))
            .collect();

        Ok(Handler { repository, static_resources })
    }


    pub async fn handle(&self, req: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
        let result = self.route(req).await;

        let response = match result {
            Ok(HTTPResponse { body, headers }) => {
                let mut builder = Response::builder()
                    .status(200);

                for (key, value) in headers {
                    builder = builder.header(key, value)
                }

                builder
                    .body(Full::from(body))
                    .unwrap()
            }
            Err(e) => {
                println!("Error: {:?}", e);
                Response::builder()
                    .status(e.status_code())
                    .body(Full::from(e.to_string()))
                    .unwrap()
            }
        };

        Ok(response)
    }

    async fn route(&self, req: Request<Incoming>) -> Result<HTTPResponse, HTTPError> {
        let method = &req.method().clone();
        let path = req.uri().path().to_string();
        let headers = req.headers().clone();
        let query_params = req.uri().query().map(|s| s.to_string());

        let body_max_size = req
            .body()
            .size_hint()
            .upper()
            .ok_or(HTTPError::MissingHeader("Content-Length"))?;

        if body_max_size > 1024 * 16 {
            return Err(HTTPError::ContentTooLarge());
        }

        let request_body: Vec<u8> = req
            .collect()
            .await
            .map_err(|e| HTTPError::CannotReadRequestBody(e))?
            .to_bytes()
            .into();

        let static_resource = self.static_resources.get(&path);

        match (method, path.as_str(), static_resource) {
            (&hyper::Method::GET, "/", _) => {
                let body = render_template(HomeTemplate {
                    current_page: "home",
                })?;

                HTTPResponse::static_html(body)
            }

            (&hyper::Method::GET, "/about", _) => {
                let body = render_template(AboutTemplate {
                    current_page: "about",
                })?;
                HTTPResponse::static_html(body)
            }

            (&hyper::Method::GET, "/contact", _) => {
                let body = render_template(ContactTemplate {
                    current_page: "contact",
                })?;
                HTTPResponse::static_html(body)
            }

            (&hyper::Method::POST, "/contact", _) => {
                if headers.get("Content-Type") != Some(&"application/x-www-form-urlencoded".parse().unwrap()) {
                    return Err(HTTPError::InvalidContentType());
                }

                let form_data: ContactFormData = serde_urlencoded::from_bytes(&request_body)
                    .map_err(|e| HTTPError::InvalidFormData(e))?;

                let timestamp = SystemTime::now();
                let name = form_data.name.try_into()
                    .map_err(|e| HTTPError::InvalidField("name", e))?;
                let email = form_data.email.try_into()
                    .map_err(|e| HTTPError::InvalidField("email", e))?;
                let contents = form_data.message.try_into()
                    .map_err(|e| HTTPError::InvalidField("message", e))?;

                let message = Message::new(timestamp, name, email, contents);

                self.repository.create(&message)
                    .await
                    .map_err(|e| HTTPError::RepositoryError(e))?;

                let body = "<p>Thank you for your message!</p>".to_string();
                HTTPResponse::static_html(body)
            }

            (&hyper::Method::GET, "/message/entries", _) => {
                let query_params = query_params
                    .unwrap_or("".to_string());

                let query: ListMessageEntriesQuery = serde_urlencoded::from_str(&query_params)
                    .map_err(|e| HTTPError::InvalidQueryParameters(e))?;

                let max_results = query.max_results.unwrap_or(10);

                let page_token = query.page_token
                    .map(|s| s.try_into())
                    .transpose()
                    .map_err(|e| HTTPError::InvalidField("page token", e))?;

                let (results, next_page_token) = self.repository.list(max_results, page_token)
                    .await
                    .map_err(|e| HTTPError::RepositoryError(e))?;

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

                let body = render_template(MessageEntriesTemplate {
                    entries: results,
                    max_results: query.max_results.unwrap_or(10),
                    has_next_page: next_page_token.is_some(),
                    next_page_token: next_page_token.map(|p| p.to_string()).unwrap_or_default(),
                })?;

                HTTPResponse::dynamic_html(body)
            }

            (&hyper::Method::GET, "/messages", _) => {
                let body = render_template(MessagesTemplate {
                    current_page: "messages",
                })?;
                HTTPResponse::static_html(body)
            }

            (&hyper::Method::GET, _, Some(resource)) => {
                Ok(HTTPResponse {
                    body: resource.contents.clone(),
                    headers: vec![
                        ("Content-Type".to_string(), resource.content_type.to_string()),
                        ("Content-Encoding".to_string(), "gzip".to_string()),
                        ("Cache-Control".to_string(), "public,max-age=31536000".to_string()),
                        ("ETag".to_string(), resource.etag.clone()),
                    ],
                })
            }

            _ =>
                Err(HTTPError::PageNotFound()),
        }
    }
}

fn render_template(template: impl askama::Template) -> Result<String, HTTPError> {
    template
        .render()
        .map_err(|e| HTTPError::TemplateRenderingIssue(e))
}


struct HTTPResponse {
    body: Vec<u8>,
    headers: Vec<(String, String)>,
}

impl HTTPResponse {
    fn dynamic_html(body: String) -> Result<HTTPResponse, HTTPError> {
        let etag = compute_etag(&body.as_bytes());
        Ok(HTTPResponse {
            body: gzip(body.into())?,
            headers: vec![
                ("Content-Type".to_string(), "text/html".to_string()),
                ("Content-Encoding".to_string(), "gzip".to_string()),
                ("ETag".to_string(), etag),
                ("Cache-Control".to_string(), "no-cache".to_string()),
            ],
        })
    }

    fn static_html(body: String) -> Result<HTTPResponse, HTTPError> {
        let etag = compute_etag(&body.as_bytes());
        Ok(HTTPResponse {
            body: gzip(body.into())?,
            headers: vec![
                ("Content-Type".to_string(), "text/html".to_string()),
                ("Content-Encoding".to_string(), "gzip".to_string()),
                ("ETag".to_string(), etag),
                ("Cache-Control".to_string(), "max-age=31536000".to_string()),
            ],
        })
    }
}


fn gzip(v: Vec<u8>) -> Result<Vec<u8>, HTTPError> {
    let mut encoder = GzEncoder::new(Vec::new(), flate2::Compression::best());

    encoder.write_all(&v)
        .map_err(|e| HTTPError::FailedToCompress(e))?;

    let compressed = encoder.finish()
        .map_err(|e| HTTPError::FailedToCompress(e))?;

    Ok(compressed)
}

struct StaticResource {
    route: String,
    contents: Vec<u8>,
    content_type: String,
    etag: String,
}

impl StaticResource {
    fn new(route: &str, compressed_contents: Vec<u8>, content_type: &str) -> Self {
        let etag = compute_etag(&compressed_contents);
        StaticResource {
            route: route.to_string(),
            contents: compressed_contents,
            content_type: content_type.to_string(),
            etag,
        }
    }

    async fn load(route: &str, file_path: &'static str, content_type: &str) -> Result<Self, StartupError> {
        let contents = fs::read(file_path)
            .await
            .map_err(|e| StartupError::CouldNotReadStaticFile(file_path, e))?;

        let mut encoder = GzEncoder::new(Vec::new(), flate2::Compression::best());

        encoder.write_all(&contents)
            .map_err(|e| StartupError::FailedToCompressStaticFile(e))?;

        let compressed_contents = encoder.finish()
            .map_err(|e| StartupError::FailedToCompressStaticFile(e))?;

        Ok(StaticResource::new(
            route,
            compressed_contents,
            content_type,
        ))
    }
}

fn compute_etag(v: &[u8]) -> String {
    let hash = xxh3_64(v);
    format!("\"{:x}\"", hash)
}