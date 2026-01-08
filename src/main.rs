use std::sync::Arc;

use anyhow::Result;
use dotenv::dotenv;

mod bot;
mod llm;
mod memory;
mod persona;
mod platform;
pub mod prompt;

use bot::Bot;
use llm::MockLLM;
use platform::{Platform, terminal::TerminalPlatform};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    // Initialize Logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Initializing AI Chatbot...");

    // 1. Initialize LLM Client
    let provider = std::env::var("LLM_PROVIDER").unwrap_or_else(|_| "mock".to_string());

    let llm_client: Arc<dyn llm::LLMClient>;
    let mut vision_client: Option<Arc<dyn llm::VisionClient>> = None;
    let mut voice_client: Option<Arc<dyn llm::VoiceClient>> = None;

    match provider.to_lowercase().as_str() {
        "deepseek" => {
            llm_client = Arc::new(
                llm::deepseek::DeepSeekClient::new().expect("Failed to init DeepSeek client"),
            );
        }
        "doubao" => {
            let client =
                Arc::new(llm::doubao::DoubaoClient::new().expect("Failed to init Doubao client"));
            llm_client = client.clone();
            vision_client = Some(client.clone() as Arc<dyn llm::VisionClient>);
            voice_client = Some(client.clone() as Arc<dyn llm::VoiceClient>);
        }
        "grok" => {
            llm_client =
                Arc::new(llm::grok::GrokClient::new().expect("Failed to init Grok client"));
        }
        _ => {
            if provider != "mock" {
                tracing::warn!("Unknown provider '{}', falling back to MockLLM", provider);
            }
            llm_client = Arc::new(MockLLM);
        }
    }

    // 2. Initialize Memory (Optional)
    let memory_type = std::env::var("MEMORY_TYPE").unwrap_or_else(|_| "none".to_string());
    let memory: Option<std::sync::Arc<dyn memory::Memory>> = match memory_type.as_str() {
        "postgres" => {
            tracing::info!("Initializing Postgres Memory...");
            let mem = memory::postgres::PostgresMemory::new()
                .await
                .expect("Failed to init Postgres memory");
            Some(std::sync::Arc::new(mem))
        }
        "redis" => {
            tracing::info!("Initializing Redis Memory...");
            let mem = memory::redis::RedisMemory::new().expect("Failed to init Redis memory");
            Some(std::sync::Arc::new(mem))
        }
        _ => None,
    };

    // 3. Initialize Persona Manager
    tracing::info!("Loading Personas...");
    let persona_manager = std::sync::Arc::new(
        persona::PersonaManager::new("avatars", "default")
            .expect("Failed to initialize Persona Manager"),
    );

    // 4. Initialize Bot Core
    let bot = std::sync::Arc::new(Bot::new(
        llm_client,
        memory,
        persona_manager,
        vision_client,
        voice_client,
    ));

    // 5. Initialize Platform Adapter
    let platform_type = std::env::var("PLATFORM").unwrap_or_else(|_| "terminal".to_string());

    if platform_type.as_str() == "onebot" {
        let platform = platform::onebot::OneBotPlatform;
        platform.run(bot).await?;
    } else {
        let platform = TerminalPlatform;
        platform.run(bot).await?;
    }

    Ok(())
}
