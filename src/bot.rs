use std::sync::Arc;

use anyhow::Result;

use crate::{
    llm::{LLMClient, VisionClient, VoiceClient},
    memory::Memory,
    persona::PersonaManager,
    prompt::{Input, Message},
};

pub struct Bot {
    llm: Arc<dyn LLMClient>,
    memory: Option<Arc<dyn Memory>>,
    persona_manager: Arc<PersonaManager>,
    vision_client: Option<Arc<dyn VisionClient>>,
    voice_client: Option<Arc<dyn VoiceClient>>,
}

impl Bot {
    pub fn new(
        llm: Arc<dyn LLMClient>,
        memory: Option<Arc<dyn Memory>>,
        persona_manager: Arc<PersonaManager>,
        vision_client: Option<Arc<dyn VisionClient>>,
        voice_client: Option<Arc<dyn VoiceClient>>,
    ) -> Self {
        Self {
            llm,
            memory,
            persona_manager,
            vision_client,
            voice_client,
        }
    }

    pub fn get_greeting(&self) -> String {
        self.persona_manager
            .get_default_persona()
            .greeting
            .clone()
            .unwrap_or_else(|| "Hello! I am ready.".to_string())
    }

    pub async fn handle_message(
        &self,
        session_id: &str,
        input: Input,
        user_id: Option<&str>,
    ) -> Result<String> {
        match input {
            Input::Text(text) => self.handle_text(session_id, &text, user_id).await,
            Input::Image(url) => {
                if let Some(vision) = &self.vision_client {
                    let analysis = vision.analyze_image(&url, "Describe this image").await?;
                    Ok(analysis)
                } else {
                    Ok("Vision capability not enabled.".to_string())
                }
            }
            Input::Audio(data) => {
                if let Some(voice) = &self.voice_client {
                    let text = voice.speech_to_text(&data).await?;
                    let response = self.handle_text(session_id, &text, user_id).await?;
                    Ok(response)
                } else {
                    Ok("Voice capability not enabled.".to_string())
                }
            }
            Input::Video(url) => {
                if let Some(vision) = &self.vision_client {
                    let analysis = vision.analyze_video(&url, "Describe this video").await?;
                    Ok(analysis)
                } else {
                    Ok("Vision capability not enabled.".to_string())
                }
            }
        }
    }

    async fn handle_text(
        &self,
        session_id: &str,
        input: &str,
        user_id: Option<&str>,
    ) -> Result<String> {
        // 1. Get Persona (Default for now)
        let persona = self.persona_manager.get_default_persona();

        // 2. Save User Message
        let user_msg = Message::user(input, user_id.map(std::string::ToString::to_string));

        if let Some(mem) = &self.memory {
            mem.add_message(session_id, user_msg.clone()).await?;
        }

        // 3. Build Context (Messages)
        let mut messages = Vec::new();

        // System Prompt (from Persona)
        messages.push(Message::system(&persona.system_prompt));

        // History
        if let Some(mem) = &self.memory {
            let history = mem.get_history(session_id).await?;
            messages.extend(history);
        } else {
            messages.push(user_msg.clone());
        }

        let response_text = self.llm.chat(&messages).await?;

        // 4. Save Assistant Message
        if let Some(mem) = &self.memory {
            let bot_msg = Message::assistant(&response_text);
            mem.add_message(session_id, bot_msg).await?;
        }

        Ok(response_text)
    }
}
