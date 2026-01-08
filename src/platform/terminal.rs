use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::{bot::Bot, platform::Platform, prompt::Input};

pub struct TerminalPlatform;

#[async_trait]
impl Platform for TerminalPlatform {
    async fn run(&self, bot: Arc<Bot>) -> Result<()> {
        println!("{}", bot.get_greeting());
        println!("(Type 'exit' to quit)");

        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut stdout = io::stdout();
        let mut input = String::new();

        loop {
            stdout.write_all(b"You: ").await?;
            stdout.flush().await?;
            input.clear();

            // read_line is cancel safe and async
            if reader.read_line(&mut input).await? == 0 {
                break; // EOF
            }

            let trimmed = input.trim();
            if trimmed.eq_ignore_ascii_case("exit") || trimmed.eq_ignore_ascii_case("quit") {
                break;
            }

            if trimmed.is_empty() {
                continue;
            }

            match bot
                .handle_message("terminal-session", Input::Text(trimmed.to_string()), None)
                .await
            {
                Ok(response) => {
                    // Use stdout for response too
                    let output = format!("{response}\n");
                    stdout.write_all(output.as_bytes()).await?;
                    stdout.flush().await?;
                }
                Err(e) => {
                    let err_msg = format!("Error: {e}\n");
                    stdout.write_all(err_msg.as_bytes()).await?;
                    stdout.flush().await?;
                }
            }
        }

        println!("Goodbye!");
        Ok(())
    }
}
