use std::net::AddrParseError;

#[derive(Debug)]
pub enum StartupError {
    CouldNotBind(std::io::Error),
    InvalidListenAddress(AddrParseError),
    CannotCreateConnectionPool(sqlx::Error),
    CouldNotServe(std::io::Error),
}


impl std::fmt::Display for StartupError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            StartupError::InvalidListenAddress(e) =>
                write!(f, "invalid listen address: {}", e),

            StartupError::CouldNotBind(e) =>
                write!(f, "could not bind TCP server: {}", e),

            StartupError::CannotCreateConnectionPool(e) =>
                write!(f, "cannot create connection pool: {}", e),

            StartupError::CouldNotServe(e) =>
                write!(f, "could not serve: {}", e),
        }
    }
}


impl std::error::Error for StartupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
