use std::env;

use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{llm::LLMClient, prompt::Message};

pub struct GrokClient {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
}

#[derive(Serialize)]
struct GrokRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
}

#[derive(Deserialize)]
struct GrokResponse {
    choices: Vec<GrokChoice>,
}

#[derive(Deserialize)]
struct GrokChoice {
    message: Message,
}

impl GrokClient {
    #[allow(clippy::unnecessary_wraps)]
    pub fn new() -> Result<Self> {
        let api_key = env::var("GROK_API_KEY").expect("GROK_API_KEY must be set");
        let model = env::var("GROK_MODEL").unwrap_or_else(|_| "grok-beta".to_string());

        Ok(Self {
            client: Client::new(),
            api_key,
            model,
            base_url: "https://api.x.ai/v1/chat/completions".to_string(),
        })
    }
}

#[async_trait]
impl LLMClient for GrokClient {
    async fn chat(&self, messages: &[Message]) -> Result<String> {
        let request = GrokRequest {
            model: self.model.clone(),
            messages: messages.to_vec(),
            stream: false,
        };

        let res = self
            .client
            .post(&self.base_url)
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await?;

        if !res.status().is_success() {
            let error_text = res.text().await?;
            return Err(anyhow::anyhow!("Grok API Error: {error_text}"));
        }

        let response_body: GrokResponse = res.json().await?;
        let content = response_body
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(content)
    }
}
