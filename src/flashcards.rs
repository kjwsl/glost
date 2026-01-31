use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flashcard {
    pub id: String,
    pub word: String,
    pub definition: String,
    pub context: Option<String>,
    pub source: Option<String>,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FlashcardList {
    pub cards: Vec<Flashcard>,
}

impl FlashcardList {
    pub fn new() -> Self {
        Self { cards: Vec::new() }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(path)?;
        let list: Self = serde_json::from_str(&content)?;
        Ok(list)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn add(&mut self, word: String, definition: String, context: Option<String>, source: Option<String>) {
        let card = Flashcard {
            id: uuid::Uuid::new_v4().to_string(),
            word,
            definition,
            context,
            source,
            added_at: Utc::now(),
        };
        self.cards.push(card);
    }
}
