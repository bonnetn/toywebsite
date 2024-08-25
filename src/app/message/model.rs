use std::fmt::Display;
use std::time::SystemTime;
use crate::app::validation;

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

impl TryFrom<String> for Name {
    type Error = validation::Error;

    fn try_from(name: String) -> Result<Self, Self::Error> {
        if name.len() < 1 {
            return Err(validation::Error::TooShort);
        }

        if name.as_bytes().len() > 255 {
            return Err(validation::Error::TooLong);
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

impl TryFrom<String> for Email {
    type Error = validation::Error;

    fn try_from(email: String) -> Result<Self, Self::Error> {
        if email.len() < 1 {
            return Err(validation::Error::TooShort);
        }

        if email.as_bytes().len() > 255 {
            return Err(validation::Error::TooLong);
        }

        if !email.contains('@') {
            return Err(validation::Error::InvalidEmail);
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

impl TryFrom<String> for Contents {
    type Error = validation::Error;

    fn try_from(message: String) -> Result<Self, Self::Error> {
        if message.len() < 1 {
            return Err(validation::Error::TooShort);
        }

        if message.as_bytes().len() > 1024 {
            return Err(validation::Error::TooLong);
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


#[derive(Debug, Copy, Clone)]
pub struct PageToken(usize);

impl PageToken {
    pub fn new(offset: usize) -> Self {
        PageToken(offset)
    }
}

impl TryFrom<String> for PageToken {
    type Error = validation::Error;

    fn try_from(token: String) -> Result<Self, Self::Error> {
        let token = token.parse()
            .map_err(|_| validation::Error::InvalidPageToken)?;

        Ok(PageToken(token))
    }
}

impl Display for PageToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let PageToken(token) = self;
        write!(f, "{}", token)
    }
}

impl PageToken {
    pub fn offset(&self) -> usize {
        let PageToken(token) = self;
        *token
    }
}