use std::{env, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{error, info};

use crate::{bot::Bot, platform::Platform, prompt::Input};

pub struct OneBotPlatform;

#[derive(Serialize)]
struct SendMessageParams {
    message_type: String,
    user_id: Option<i64>,
    group_id: Option<i64>,
    message: String,
}

#[derive(Serialize)]
struct ApiCall {
    action: String,
    params: SendMessageParams,
}

// Minimal event structure
#[derive(Deserialize, Debug)]
struct Event {
    post_type: String,
    message_type: Option<String>,
    #[allow(dead_code)]
    sub_type: Option<String>,
    user_id: Option<i64>,
    group_id: Option<i64>,
    #[allow(dead_code)]
    message: Option<String>,
    raw_message: Option<String>,
    #[allow(dead_code)]
    sender: Option<Sender>,
}

#[derive(Deserialize, Debug)]
struct Sender {
    #[allow(dead_code)]
    nickname: Option<String>,
}

#[async_trait]
impl Platform for OneBotPlatform {
    async fn run(&self, bot: Arc<Bot>) -> Result<()> {
        let ws_url =
            env::var("ONEBOT_WS_URL").unwrap_or_else(|_| "ws://127.0.0.1:6700".to_string());

        info!("Connecting to OneBot at {}...", ws_url);

        let (ws_stream, _) = connect_async(&ws_url).await?;
        info!("Connected to OneBot!");

        let (mut write, mut read) = ws_stream.split();

        while let Some(msg) = read.next().await {
            let msg = match msg {
                Ok(m) => m,
                Err(e) => {
                    error!("Error reading WS message: {}", e);
                    continue;
                }
            };

            if let Message::Text(text) = msg {
                // Parse event
                if let Ok(event) = serde_json::from_str::<Event>(&text) {
                    // Filter for normal messages
                    if event.post_type == "message" {
                        let raw_msg = event.raw_message.unwrap_or_default();
                        let user_id = event.user_id.unwrap_or(0);
                        let group_id = event.group_id;
                        let msg_type = event.message_type.unwrap_or("private".to_string());

                        // Session ID: "onebot:private:123" or "onebot:group:456:123"
                        let session_id = if let Some(gid) = group_id {
                            format!("onebot:group:{gid}:{user_id}")
                        } else {
                            format!("onebot:private:{user_id}")
                        };

                        info!("Received message from {}: {}", session_id, raw_msg);

                        // Process with Bot
                        // Note: We might want to filter self-messages if the bridge echoes them,
                        // but standard OneBot doesn't usually echo unless configured.

                        match bot
                            .handle_message(
                                &session_id,
                                Input::Text(raw_msg.clone()),
                                Some(&user_id.to_string()),
                            )
                            .await
                        {
                            Ok(reply) => {
                                // Send Reply
                                let api_call = ApiCall {
                                    action: "send_msg".to_string(),
                                    params: SendMessageParams {
                                        message_type: msg_type,
                                        user_id: if group_id.is_none() {
                                            Some(user_id)
                                        } else {
                                            None
                                        }, // For private
                                        group_id, // For group
                                        message: reply,
                                    },
                                };

                                let json = serde_json::to_string(&api_call)?;
                                write.send(Message::Text(json)).await?;
                            }
                            Err(e) => error!("Bot error: {}", e),
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
