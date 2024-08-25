use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    TooShort,
    TooLong,
    InvalidEmail,
    InvalidPageToken,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::TooShort => write!(f, "too short"),
            Error::TooLong => write!(f, "too long"),
            Error::InvalidEmail => write!(f, "invalid email"),
            Error::InvalidPageToken => write!(f, "invalid page token"),
        }
    }
}

