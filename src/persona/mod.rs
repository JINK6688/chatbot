use std::{collections::HashMap, fs};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    pub name: String,
    pub description: String,
    pub system_prompt: String,
    pub greeting: Option<String>,
}

pub struct PersonaManager {
    personas: HashMap<String, Persona>,
    default_persona: String,
}

impl PersonaManager {
    pub fn new(avatars_dir: &str, default_name: &str) -> Result<Self> {
        let mut personas = HashMap::new();

        // Read all json files in avatars_dir
        let entries = fs::read_dir(avatars_dir)
            .context(format!("Failed to read avatars directory: {avatars_dir}"))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path)
                    .context(format!("Failed to read persona file: {}", path.display()))?;
                let persona: Persona = serde_json::from_str(&content)
                    .context(format!("Failed to parse persona file: {}", path.display()))?;

                let key = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(&persona.name) // Fallback to inner name if file stem fails? Or just use inner name? The prompt
                    // says "use filename".
                    .to_string();

                personas.insert(key, persona);
            }
        }

        if personas.is_empty() {
            // If no files, create a fallback in memory
            let fallback = Persona {
                name: "default".to_string(),
                description: "Default AI Assistant".to_string(),
                system_prompt: "You are a helpful AI assistant.".to_string(),
                greeting: Some("Hello! How can I help you?".to_string()),
            };
            personas.insert("default".to_string(), fallback);
        }

        Ok(Self {
            personas,
            default_persona: default_name.to_string(),
        })
    }

    pub fn get_default_persona(&self) -> &Persona {
        self.personas
            .get(&self.default_persona)
            .or_else(|| self.personas.values().next())
            .expect("PersonaManager should have at least one persona")
    }
}
