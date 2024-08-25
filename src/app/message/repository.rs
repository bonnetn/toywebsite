use std::fmt::Display;
use serde_json::Deserializer;
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use crate::app::message::model::{Message, PageToken};
use crate::app::message::repository::dto::MessageDTO;
use crate::app::validation;

const MAX_RESULTS: usize = 100;

#[derive(Debug)]
pub struct Repository {
    filename: String,
}

#[derive(Debug)]
pub enum Error {
    CannotAppendDatabaseFile(std::io::Error),
    CannotDeserializeMessageFromDatabase(serde_json::Error),
    CannotMapObjectFromDatabase(&'static str, validation::Error),
    CannotReadDatabaseFile(std::io::Error),
    CannotSerializeMessageToDatabase(serde_json::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::CannotSerializeMessageToDatabase(e) =>
                write!(f, "cannot serialize message to database: {}", e),
            Error::CannotAppendDatabaseFile(e) =>
                write!(f, "cannot append to database file: {}", e),
            Error::CannotReadDatabaseFile(e) =>
                write!(f, "cannot read database file: {}", e),
            Error::CannotDeserializeMessageFromDatabase(e) =>
                write!(f, "cannot deserialize message from database: {}", e),
            Error::CannotMapObjectFromDatabase(field, error) =>
                write!(f, "cannot map object from database: field {}: {}", field, error),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;


impl Repository {
    pub fn new(filename: &str) -> Self {
        Repository {
            filename: filename.to_string(),
        }
    }

    pub async fn create_message(&self, msg: &Message) -> Result<()> {
        let msg_dto: MessageDTO = msg.into();
        let mut msg_json = serde_json::to_string(&msg_dto)
            .map_err(|e| Error::CannotSerializeMessageToDatabase(e))?;
        msg_json.push('\n');

        let mut file = File::options()
            .create(true)
            .append(true)
            .open(&self.filename)
            .await
            .map_err(|e| Error::CannotAppendDatabaseFile(e))?;

        file
            .write_all(msg_json.as_bytes())
            .await
            .map_err(|e| Error::CannotAppendDatabaseFile(e))?;

        Ok(())
    }


    pub async fn list_messages(&self, max_results: usize, page_token: Option<PageToken>) -> Result<(Vec<Message>, Option<PageToken>)> {
        let max_results = match max_results {
            v if v == 0 =>
                MAX_RESULTS,
            v if v > MAX_RESULTS =>
                MAX_RESULTS,
            v =>
                v
        };

        let page_token: usize = match page_token {
            Some(token) =>
                token.offset(),
            None =>
                0,
        };

        let database_contents = fs::read(&self.filename)
            .await
            .map_err(|e| Error::CannotReadDatabaseFile(e))?;

        let dtos = Deserializer::from_slice(&database_contents)
            .into_iter::<MessageDTO>()
            .skip(page_token)
            .take(max_results)
            .collect::<std::result::Result<Vec<MessageDTO>, _>>()
            .map_err(|e| Error::CannotDeserializeMessageFromDatabase(e))?;

        let messages = dtos
            .into_iter()
            .map(|dto| dto.try_into())
            .collect::<Result<Vec<Message>>>()?;

        let next_page_token = if messages.len() < max_results {
            None
        } else {
            Some(PageToken::new(page_token + max_results))
        };

        Ok((messages, next_page_token))
    }
}

mod dto {
    use serde::{Deserialize, Serialize};
    use crate::app::message::Message;
    use crate::app::message::repository::Error;

    #[derive(Debug, Deserialize, Serialize)]
    pub struct MessageDTO {
        timestamp: u128,
        name: String,
        email: String,
        contents: String,
    }

    impl From<&Message> for MessageDTO {
        fn from(msg: &Message) -> Self {
            MessageDTO {
                timestamp: msg.timestamp().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),
                name: msg.name().to_string(),
                email: msg.email().to_string(),
                contents: msg.contents().to_string(),
            }
        }
    }

    impl TryInto<Message> for MessageDTO {
        type Error = Error;

        fn try_into(self) -> Result<Message, Self::Error> {
            // TODO: Error?
            let timestamp = self.timestamp.try_into().unwrap();

            let timestamp = std::time::UNIX_EPOCH + std::time::Duration::from_nanos(timestamp);

            let name = self.name
                .try_into()
                .map_err(|e| Error::CannotMapObjectFromDatabase("name", e))?;

            let email = self.email
                .try_into()
                .map_err(|e| Error::CannotMapObjectFromDatabase("email", e))?;

            let contents = self.contents
                .try_into()
                .map_err(|e| Error::CannotMapObjectFromDatabase("contents", e))?;

            Ok(Message::new(timestamp, name, email, contents))
        }
    }
}
