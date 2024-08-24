use std::net::AddrParseError;

#[derive(Debug)]
pub enum StartupError {
    InvalidListenAddress(AddrParseError),
    CouldNotBind(std::io::Error),
    CouldNotAcceptConnection(std::io::Error),
    CouldNotReadStaticFile(&'static str, std::io::Error),
    FailedToCompressStaticFile(std::io::Error),
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
    BadRequest(&'static str),
    InvalidContentType(),
    ContentTooLarge(),
    PageNotFound(),
    TemplateRenderingIssue(askama::Error),
    FailedToCompress(std::io::Error),
    CannotReadRequestBody(hyper::Error),
    CannotReadDatabaseFile(std::io::Error),
    CannotAppendDatabaseFile(std::io::Error),
    CannotSerializeMessageToDatabase(serde_json::Error),
    CannotDeserializeMessageFromDatabase(serde_json::Error),
}

impl std::fmt::Display for HTTPError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HTTPError::BadRequest(e) =>
                write!(f, "bad request: {}", e),
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
            HTTPError::CannotReadDatabaseFile(e) =>
                write!(f, "could not read database file: {}", e),
            HTTPError::CannotAppendDatabaseFile(e) =>
                write!(f, "could not append database file: {}", e),
            HTTPError::CannotSerializeMessageToDatabase(e) =>
                write!(f, "could not serialize message to database: {}", e),
            HTTPError::CannotDeserializeMessageFromDatabase(e) =>
                write!(f, "could not deserialize message from database: {}", e),
        }
    }
}

impl std::error::Error for HTTPError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}