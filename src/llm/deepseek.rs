use std::env;

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{llm::LLMClient, prompt::Message};

pub struct DeepSeekClient {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl DeepSeekClient {
    pub fn new() -> Result<Self> {
        let api_key = env::var("DEEPSEEK_API_KEY").context("DEEPSEEK_API_KEY not set")?;
        let model = env::var("DEEPSEEK_MODEL").unwrap_or_else(|_| "deepseek-chat".to_string());

        Ok(Self {
            api_key,
            model,
            client: reqwest::Client::new(),
        })
    }
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
}

// Local Message struct removed in favor of crate::prompt::Message usage.

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

#[async_trait]
impl LLMClient for DeepSeekClient {
    async fn chat(&self, messages: &[Message]) -> Result<String> {
        let request = ChatRequest {
            model: self.model.clone(),
            messages: messages.to_vec(),
        };

        let response = self
            .client
            .post("https://api.deepseek.com/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("DeepSeek API error: {error_text}");
        }

        let chat_response: ChatResponse = response.json().await?;

        chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .context("No response choice from DeepSeek API")
    }
}
