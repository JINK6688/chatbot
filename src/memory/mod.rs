use anyhow::Result;
use async_trait::async_trait;

use crate::prompt::Message;

#[async_trait]
pub trait Memory: Send + Sync {
    // Retrieve recent history for context
    async fn get_history(&self, session_id: &str) -> Result<Vec<Message>>;

    // Save a new message to the history
    async fn add_message(&self, session_id: &str, message: Message) -> Result<()>;
}

pub mod postgres;
pub mod redis;
