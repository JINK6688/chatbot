use std::env;

use anyhow::Result;
use async_trait::async_trait;
use redis::AsyncCommands;

use crate::{memory::Memory, prompt::Message};

pub struct RedisMemory {
    client: redis::Client,
}

impl RedisMemory {
    pub fn new() -> Result<Self> {
        let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
        let client = redis::Client::open(redis_url)?;
        Ok(Self { client })
    }
}

#[async_trait]
impl Memory for RedisMemory {
    async fn get_history(&self, session_id: &str) -> Result<Vec<Message>> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("chat:{session_id}");

        let raw_messages: Vec<String> = conn.lrange(&key, 0, -1).await?;

        let mut messages = Vec::new();
        for raw in raw_messages {
            if let Ok(msg) = serde_json::from_str::<Message>(&raw) {
                messages.push(msg);
            }
        }

        Ok(messages)
    }

    async fn add_message(&self, session_id: &str, message: Message) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("chat:{session_id}");

        let json = serde_json::to_string(&message)?;
        let _: () = conn.rpush(&key, json).await?;
        // Optional: Set expire
        let _: () = conn.expire(&key, 3600 * 24).await?; // 24 hours

        Ok(())
    }
}
