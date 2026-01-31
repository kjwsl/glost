use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use chrono::{DateTime, Duration, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flashcard {
    pub id: String,
    pub word: String,
    pub definition: String,
    pub context: Option<String>,
    pub source: Option<String>,
    pub added_at: DateTime<Utc>,
    
    // SRS fields
    #[serde(default = "default_next_review")]
    pub next_review_at: DateTime<Utc>,
    #[serde(default = "default_interval")]
    pub interval_days: u32,
    #[serde(default = "default_ease")]
    pub ease_factor: f32,
    #[serde(default)]
    pub repetitions: u32,
}

fn default_next_review() -> DateTime<Utc> {
    Utc::now()
}

fn default_interval() -> u32 {
    0
}

fn default_ease() -> f32 {
    2.5
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
            next_review_at: Utc::now(),
            interval_days: 0,
            ease_factor: 2.5,
            repetitions: 0,
        };
        self.cards.push(card);
    }

    pub fn review_card(&mut self, card_id: &str, quality: u8) -> Option<()> {
        let card = self.cards.iter_mut().find(|c| c.id == card_id)?;
        
        // SuperMemo-2 Algorithm
        // quality: 0-5
        // 0: blackout, 1: incorrect response, 2: incorrect (but familiar), 
        // 3: correct (difficulty), 4: correct (hesitation), 5: perfect
        
        if quality >= 3 {
            if card.repetitions == 0 {
                card.interval_days = 1;
            } else if card.repetitions == 1 {
                card.interval_days = 6;
            } else {
                card.interval_days = (card.interval_days as f32 * card.ease_factor).round() as u32;
            }
            card.repetitions += 1;
        } else {
            card.repetitions = 0;
            card.interval_days = 1;
        }

        card.ease_factor = card.ease_factor + (0.1 - (5.0 - quality as f32) * (0.08 + (5.0 - quality as f32) * 0.02));
        if card.ease_factor < 1.3 {
            card.ease_factor = 1.3;
        }

        card.next_review_at = Utc::now() + Duration::days(card.interval_days as i64);
        Some(())
    }
}
