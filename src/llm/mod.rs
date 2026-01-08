use anyhow::Result;
use async_trait::async_trait;

use crate::prompt::Message;

#[async_trait]
pub trait LLMClient: Send + Sync {
    async fn chat(&self, messages: &[Message]) -> Result<String>;
}

#[async_trait]
pub trait VisionClient: Send + Sync {
    async fn analyze_image(&self, image_url_or_base64: &str, prompt: &str) -> Result<String>;
    async fn analyze_video(&self, video_url_or_data: &str, prompt: &str) -> Result<String>;
}

#[async_trait]
pub trait VoiceClient: Send + Sync {
    async fn speech_to_text(&self, audio_data: &[u8]) -> Result<String>;
    #[allow(dead_code)]
    async fn text_to_speech(&self, text: &str) -> Result<Vec<u8>>;
}

pub struct MockLLM;

#[async_trait]
impl LLMClient for MockLLM {
    async fn chat(&self, messages: &[Message]) -> Result<String> {
        // Simple echo/dummy response for verification
        let last_msg = messages
            .last()
            .map(|m| m.content.clone())
            .unwrap_or_default();
        Ok(format!("MockAI: I received your message: '{last_msg}'"))
    }
}

pub mod deepseek;
pub mod doubao;
pub mod grok;
