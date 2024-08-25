use std::net::AddrParseError;
use crate::app::message::repository;
use crate::app::validation;

#[derive(Debug)]
pub enum StartupError {
    CouldNotAcceptConnection(std::io::Error),
    CouldNotBind(std::io::Error),
    CouldNotReadStaticFile(&'static str, std::io::Error),
    FailedToCompressStaticFile(std::io::Error),
    InvalidListenAddress(AddrParseError),
    CannotCreateRepository(crate::app::message::repository::sqlite::Error),
}


impl std::fmt::Display for StartupError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            StartupError::InvalidListenAddress(e) =>
                write!(f, "invalid listen address: {}", e),

            StartupError::CouldNotBind(e) =>
                write!(f, "could not bind TCP server: {}", e),

            StartupError::CouldNotAcceptConnection(e) =>
                write!(f, "could not accept connection: {}", e),

            StartupError::CouldNotReadStaticFile(file, e) =>
                write!(f, "could not read static file {}: {}", file, e),

            StartupError::FailedToCompressStaticFile(e) =>
                write!(f, "compression issue: {}", e),

            StartupError::CannotCreateRepository(e) =>
                write!(f, "cannot create repository: {}", e),
        }
    }
}


impl std::error::Error for StartupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[derive(Debug)]
pub enum HTTPError {
    CannotReadRequestBody(hyper::Error),
    ContentTooLarge(),
    FailedToCompress(std::io::Error),
    InvalidContentType(),
    InvalidField(&'static str, validation::Error),
    InvalidFormData(serde::de::value::Error),
    InvalidQueryParameters(serde_urlencoded::de::Error),
    MissingHeader(&'static str),
    PageNotFound(),
    RepositoryError(repository::Error),
    TemplateRenderingIssue(askama::Error),
}

impl HTTPError {
    pub fn status_code(&self) -> hyper::StatusCode {
        match self {
            HTTPError::CannotReadRequestBody(_) => hyper::StatusCode::INTERNAL_SERVER_ERROR,
            HTTPError::ContentTooLarge() => hyper::StatusCode::PAYLOAD_TOO_LARGE,
            HTTPError::FailedToCompress(_) => hyper::StatusCode::INTERNAL_SERVER_ERROR,
            HTTPError::InvalidContentType() => hyper::StatusCode::UNSUPPORTED_MEDIA_TYPE,
            HTTPError::InvalidField(_, _) => hyper::StatusCode::BAD_REQUEST,
            HTTPError::InvalidFormData(_) => hyper::StatusCode::BAD_REQUEST,
            HTTPError::InvalidQueryParameters(_) => hyper::StatusCode::BAD_REQUEST,
            HTTPError::MissingHeader(_) => hyper::StatusCode::BAD_REQUEST,
            HTTPError::PageNotFound() => hyper::StatusCode::NOT_FOUND,
            HTTPError::RepositoryError(_) => hyper::StatusCode::INTERNAL_SERVER_ERROR,
            HTTPError::TemplateRenderingIssue(_) => hyper::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl std::fmt::Display for HTTPError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HTTPError::InvalidContentType() =>
                write!(f, "invalid content type"),
            HTTPError::ContentTooLarge() =>
                write!(f, "content too large"),
            HTTPError::PageNotFound() =>
                write!(f, "not found"),
            HTTPError::TemplateRenderingIssue(e) =>
                write!(f, "template rendering issue: {}", e),
            HTTPError::FailedToCompress(e) =>
                write!(f, "compression issue: {}", e),
            HTTPError::CannotReadRequestBody(e) =>
                write!(f, "could not read request body: {}", e),
            HTTPError::InvalidField(field, err) =>
                write!(f, "invalid field {}: {}", field, err),
            HTTPError::RepositoryError(e) =>
                write!(f, "repository error: {}", e),
            HTTPError::InvalidFormData(e) =>
                write!(f, "invalid form data: {}", e),
            HTTPError::InvalidQueryParameters(e) =>
                write!(f, "invalid query parameters: {}", e),
            HTTPError::MissingHeader(name) =>
                write!(f, "missing {} header", name),
        }
    }
}

impl std::error::Error for HTTPError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}