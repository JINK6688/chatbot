use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Input {
    Text(String),
    Image(String),  // URL or Base64 or Path
    Audio(Vec<u8>), // Raw bytes
    Video(String),  // URL or Path
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

impl Message {
    #[must_use]
    pub fn new(role: &str, content: &str, user_id: Option<String>) -> Self {
        Self {
            role: role.to_string(),
            content: content.to_string(),
            user_id,
        }
    }

    #[must_use]
    pub fn system(content: &str) -> Self {
        Self::new("system", content, None)
    }

    #[must_use]
    pub fn user(content: &str, user_id: Option<String>) -> Self {
        Self::new("user", content, user_id)
    }

    #[must_use]
    pub fn assistant(content: &str) -> Self {
        Self::new("assistant", content, None)
    }
}
