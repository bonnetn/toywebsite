use async_trait::async_trait;
use crate::app::message::{Message, PageToken};

pub mod sqlite;
pub mod json;


pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;

#[async_trait]
pub trait Repository: Send + Sync {
    async fn create(&self, message: &Message) -> Result<()>;
    async fn list(&self, max_results: usize, page_token: Option<PageToken>) -> Result<(Vec<Message>, Option<PageToken>)>;
}
