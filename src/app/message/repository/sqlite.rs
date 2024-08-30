use std::fmt::Display;
use async_trait::async_trait;
use sqlx::{Pool, Sqlite, SqlitePool};
use crate::app::message::{Message, repository};
use crate::app::message::model::PageToken;
use crate::app::message::repository::{Repository};
use crate::app::message::repository::sqlite::dto::MessageDTO;
use crate::app::validation;

#[derive(Clone, Debug)]
pub struct SQLiteRepository {
    pool: SqlitePool,
}


impl SQLiteRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        SQLiteRepository { pool }
    }
}

#[async_trait]
impl Repository for SQLiteRepository {
    async fn create(&self, message: &Message) -> repository::Result<()> {
        let timestamp: chrono::DateTime<chrono::Utc> = message.timestamp().into();
        let name: String = message.name().to_string();
        let email: String = message.email().to_string();
        let contents: String = message.contents().to_string();

        sqlx::query("
                INSERT INTO message (timestamp, name, email, contents)
                VALUES (?1, ?2, ?3, ?4)
            ")
            .bind(timestamp)
            .bind(name)
            .bind(email)
            .bind(contents)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::SqlxError(e))?;

        Ok(())
    }

    async fn list(&self, max_results: usize, page_token: Option<PageToken>) -> repository::Result<(Vec<Message>, Option<PageToken>)> {
        let page_token: Option<i64> = page_token.map(|t| t.offset().try_into().unwrap());
        let max_results: i64 = max_results.try_into().unwrap();

        let rows: Vec<MessageDTO> = sqlx::query_as("
                SELECT id, timestamp, name, email, contents
                FROM message
                WHERE (?1 IS NULL OR id > ?1)
                ORDER BY id
                LIMIT ?2
            ")
            .bind(page_token)
            .bind(max_results)
            .fetch_all(&self.pool)
            .await
            .unwrap();

        let next_page_token = if rows.len() < max_results as usize {
            None
        } else {
            rows
                .last()
                .map(|r| {
                    let id = r.id().try_into().unwrap();
                    PageToken::new(id)
                })
        };

        let msgs = rows.into_iter()
            .map(|r| r.try_into())
            .collect::<Result<Vec<Message>>>()?;

        Ok((msgs, next_page_token))
    }
}

mod dto {
    use crate::app::message::Message;
    use chrono::Utc;
    use crate::app::message::repository::sqlite::Error;
    use crate::app::message::repository::sqlite::Error::CouldNotMapDatabaseObject;

    #[derive(sqlx::FromRow)]
    pub struct MessageDTO {
        id: i64,
        timestamp: chrono::DateTime<Utc>,
        name: String,
        email: String,
        contents: String,
    }

    impl MessageDTO {
        pub fn id(&self) -> i64 {
            self.id
        }
    }

    impl TryInto<Message> for MessageDTO {
        type Error = Error;

        fn try_into(self) -> Result<Message, Self::Error> {
            let timestamp = self.timestamp.into();
            let name = self.name
                .try_into()
                .map_err(|e| CouldNotMapDatabaseObject(e))?;
            let email = self.email
                .try_into()
                .map_err(|e| CouldNotMapDatabaseObject(e))?;

            let contents = self.contents
                .try_into()
                .map_err(|e| CouldNotMapDatabaseObject(e))?;

            Ok(Message::new(timestamp, name, email, contents))
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    SqlxError(sqlx::Error),
    CouldNotMapDatabaseObject(validation::Error),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::SqlxError(e) => Some(e),
            Error::CouldNotMapDatabaseObject(e) => Some(e),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::SqlxError(e) =>
                write!(f, "sqlx error: {}", e),
            Error::CouldNotMapDatabaseObject(e) =>
                write!(f, "could not map database object: {}", e),
        }
    }
}
