use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use crate::bot::Bot;

#[async_trait]
pub trait Platform: Send + Sync {
    async fn run(&self, bot: Arc<Bot>) -> Result<()>;
}

pub mod onebot;
pub mod terminal;
