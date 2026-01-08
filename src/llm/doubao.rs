use std::env;

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{llm::LLMClient, prompt::Message};

pub struct DoubaoClient {
    api_key: String,
    model_endpoint: String,
    vision_endpoint: Option<String>,
    #[allow(dead_code)]
    asr_endpoint: Option<String>,
    #[allow(dead_code)]
    tts_endpoint: Option<String>,
    client: reqwest::Client,
}

impl DoubaoClient {
    pub fn new() -> Result<Self> {
        let api_key = env::var("DOUBAO_API_KEY").context("DOUBAO_API_KEY not set")?;
        let model_endpoint = env::var("DOUBAO_MODEL")
            .context("DOUBAO_MODEL not set (this should be the endpoint ID)")?;

        let vision_endpoint = env::var("DOUBAO_VISION_MODEL").ok();
        let asr_endpoint = env::var("DOUBAO_ASR_MODEL").ok();
        let tts_endpoint = env::var("DOUBAO_TTS_MODEL").ok();

        Ok(Self {
            api_key,
            model_endpoint,
            vision_endpoint,
            asr_endpoint,
            tts_endpoint,
            client: reqwest::Client::new(),
        })
    }
}

// Doubao API (Ark) is OpenAI compatible for chat completions
#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

#[async_trait]
impl LLMClient for DoubaoClient {
    async fn chat(&self, messages: &[Message]) -> Result<String> {
        let request = ChatRequest {
            model: self.model_endpoint.clone(),
            messages: messages.to_vec(),
        };

        // Ark/Volcengine endpoint
        let response = self
            .client
            .post("https://ark.cn-beijing.volces.com/api/v3/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Doubao/Ark API error: {error_text}");
        }

        let chat_response: ChatResponse = response.json().await?;

        chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .context("No response choice from Doubao API")
    }
}

use serde_json::json;

use crate::llm::{VisionClient, VoiceClient};

#[async_trait]
impl VisionClient for DoubaoClient {
    async fn analyze_image(&self, image_url: &str, prompt: &str) -> Result<String> {
        let endpoint = self
            .vision_endpoint
            .as_ref()
            .context("Vision endpoint not configured")?;

        // Construct request for Vision Model (similar to Chat but with image_url)
        let messages = vec![json!({
            "role": "user",
            "content": [
                {"type": "text", "text": prompt},
                {"type": "image_url", "image_url": {"url": image_url}}
            ]
        })];

        let request = json!({
            "model": endpoint,
            "messages": messages
        });

        let response = self
            .client
            .post("https://ark.cn-beijing.volces.com/api/v3/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Doubao Vision API error: {error_text}");
        }

        let body: serde_json::Value = response.json().await?;
        let content = body["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        Ok(content)
    }

    async fn analyze_video(&self, _video_url: &str, _prompt: &str) -> Result<String> {
        // Placeholder
        anyhow::bail!("Video analysis not yet implemented")
    }
}

#[async_trait]
impl VoiceClient for DoubaoClient {
    async fn speech_to_text(&self, _audio_data: &[u8]) -> Result<String> {
        // Placeholder
        anyhow::bail!("ASR not fully implemented yet")
    }

    async fn text_to_speech(&self, _text: &str) -> Result<Vec<u8>> {
        // Placeholder
        anyhow::bail!("TTS not fully implemented yet")
    }
}
