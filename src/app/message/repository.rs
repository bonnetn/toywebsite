use serde_json::Deserializer;
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use crate::app::error::{HTTPError};
use crate::app::message::model::{Message};
use crate::app::message::repository::dto::MessageDTO;

const MAX_RESULTS: usize = 100;

#[derive(Debug)]
pub struct Repository {
    filename: String,
}


impl Repository {
    pub fn new(filename: &str) -> Self {
        Repository {
            filename: filename.to_string(),
        }
    }

    pub async fn create_contact_entry(&self, msg: &Message) -> Result<(), HTTPError> {
        let msg_dto: MessageDTO = msg.into();
        let mut msg_json = serde_json::to_string(&msg_dto)
            .map_err(|e| HTTPError::CannotSerializeMessageToDatabase(e))?;
        msg_json.push('\n');

        let mut file = File::options()
            .create(true)
            .append(true)
            .open(&self.filename)
            .await
            .map_err(|e| HTTPError::CannotAppendDatabaseFile(e))?;

        file
            .write_all(msg_json.as_bytes())
            .await
            .map_err(|e| HTTPError::CannotAppendDatabaseFile(e))?;

        Ok(())
    }


    pub async fn list_contact_entries(&self, max_results: usize, page_token: &Option<String>) -> Result<(Vec<Message>, Option<String>), HTTPError> {
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
                token
                    .parse()
                    .map_err(|_| HTTPError::BadRequest("Invalid page token"))?,
            None =>
                0,
        };

        let database_contents = fs::read(&self.filename)
            .await
            .map_err(|e| HTTPError::CannotReadDatabaseFile(e))?;

        let dtos = Deserializer::from_slice(&database_contents)
            .into_iter::<MessageDTO>()
            .skip(page_token)
            .take(max_results)
            .collect::<Result<Vec<MessageDTO>, _>>()
            .map_err(|e| HTTPError::CannotDeserializeMessageFromDatabase(e))?;

        let messages = dtos
            .into_iter()
            .map(|dto| dto.try_into())
            .collect::<Result<Vec<Message>, _>>()?;

        let next_page_token = if messages.len() < max_results {
            None
        } else {
            Some(format!("{}", page_token + max_results))
        };

        Ok((messages, next_page_token))
    }
}

mod dto {
    use serde::{Deserialize, Serialize};
    use crate::app::error::HTTPError;
    use crate::app::message::{Contents, Email, Message, Name};

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
        type Error = HTTPError;

        fn try_into(self) -> Result<Message, Self::Error> {
            let timestamp = self.timestamp.try_into().unwrap();
            let timestamp = std::time::UNIX_EPOCH + std::time::Duration::from_nanos(timestamp);
            let name = Name::new(self.name)?;
            let email = Email::new(self.email)?;
            let contents = Contents::new(self.contents)?;

            Ok(Message::new(timestamp, name, email, contents))
        }
    }
}


