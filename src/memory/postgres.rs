use std::env;

use anyhow::Result;
use async_trait::async_trait;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

use crate::{memory::Memory, prompt::Message};

pub struct PostgresMemory {
    pool: Pool<Postgres>,
}

impl PostgresMemory {
    pub async fn new() -> Result<Self> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?;

        // Initialize schema
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS messages (
                id SERIAL PRIMARY KEY,
                session_id VARCHAR NOT NULL,
                role VARCHAR NOT NULL,
                content TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl Memory for PostgresMemory {
    async fn get_history(&self, session_id: &str) -> Result<Vec<Message>> {
        let rows = sqlx::query_as::<_, MessageRecord>(
            "SELECT role, content, user_id FROM messages WHERE session_id = $1 ORDER BY created_at ASC",
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| Message {
                role: r.role,
                content: r.content,
                user_id: r.user_id,
            })
            .collect())
    }

    async fn add_message(&self, session_id: &str, message: Message) -> Result<()> {
        sqlx::query(
            "INSERT INTO messages (session_id, role, content, user_id) VALUES ($1, $2, $3, $4)",
        )
        .bind(session_id)
        .bind(&message.role)
        .bind(&message.content)
        .bind(&message.user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct MessageRecord {
    role: String,
    content: String,
    user_id: Option<String>,
}
