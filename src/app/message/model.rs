use std::fmt::Display;
use std::time::SystemTime;
use crate::app::error::HTTPError;

#[derive(Debug, Clone)]
pub struct Message {
    timestamp: SystemTime,
    name: Name,
    email: Email,
    contents: Contents,
}

impl Message {
    pub fn new(timestamp: SystemTime, name: Name, email: Email, contents: Contents) -> Self {
        Message {
            timestamp,
            name,
            email,
            contents,
        }
    }

    pub fn timestamp(&self) -> SystemTime {
        self.timestamp
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn email(&self) -> &Email {
        &self.email
    }

    pub fn contents(&self) -> &Contents {
        &self.contents
    }
}

#[derive(Debug, Clone)]
pub struct Name(String);

impl Name {
    pub fn new(name: String) -> Result<Self, HTTPError> {
        if name.len() < 1 {
            return Err(HTTPError::BadRequest("name is too short"));
        }

        if name.as_bytes().len() > 255 {
            return Err(HTTPError::BadRequest("name is too long"));
        }

        Ok(Name(name))
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Name(name) = self;
        write!(f, "{}", name)
    }
}

#[derive(Debug, Clone)]
pub struct Email(String);

impl Email {
    pub fn new(email: String) -> Result<Self, HTTPError> {
        if email.len() < 1 {
            return Err(HTTPError::BadRequest("email is too short"));
        }

        if email.as_bytes().len() > 255 {
            return Err(HTTPError::BadRequest("email is too long"));
        }

        if !email.contains('@') {
            return Err(HTTPError::BadRequest("email is invalid"));
        }

        Ok(Email(email))
    }
}

impl Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Email(email) = self;
        write!(f, "{}", email)
    }
}

#[derive(Debug, Clone)]
pub struct Contents(String);

impl Contents {
    pub fn new(message: String) -> Result<Self, HTTPError> {
        if message.len() < 1 {
            return Err(HTTPError::BadRequest("message is too short"));
        }

        if message.as_bytes().len() > 1024 {
            return Err(HTTPError::BadRequest("message is too long"));
        }

        Ok(Contents(message))
    }
}

impl Display for Contents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Contents(message) = self;
        write!(f, "{}", message)
    }
}
