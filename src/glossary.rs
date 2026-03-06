use crate::{Language, kaikki};
use std::{collections::HashSet, str::FromStr};

pub type Glossary = HashSet<WordEntry>;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WordEntry {
    pub word: String,
    pub meaning: String,
    pub pos: POS,
    pub lang: Language,
    pub frequency: usize,
}

impl WordEntry {
    pub fn new(word: String, meaning: String, pos: POS, lang: Language, frequency: usize) -> Self {
        Self {
            word,
            meaning,
            pos,
            lang,
            frequency,
        }
    }

    pub fn from_kaikki_entry(entry: kaikki::Entry, frequency: usize) -> Option<Self> {
        let pos = entry.pos.parse::<POS>().unwrap_or(POS::Other);
        let lang = Language::from_str(&entry.lang).ok()?;

        let meaning = entry
            .senses
            .iter()
            .filter_map(|s| match &s.glosses {
                Some(g) => Some(g.join("; ")),
                None => None,
            })
            .collect::<Vec<String>>()
            .join("; ");

        Some(Self {
            word: entry.word,
            meaning,
            pos,
            lang,
            frequency,
        })
    }

    /// Get a key for merging entries (word + lang, excluding POS, frequency and meaning)
    pub fn merge_key(&self) -> (String, Language) {
        (self.word.clone(), self.lang)
    }

    /// Merge this entry with another, combining frequencies and meanings from different POS
    pub fn merge_with(&mut self, other: &WordEntry) {
        self.frequency += other.frequency;

        // If this is the first merge (meaning is in original format), convert it
        if !self.meaning.contains(" | ") {
            let formatted_meaning = if self.meaning.is_empty() {
                String::new()
            } else {
                format!("*{}*: {}", self.pos, self.meaning)
            };
            self.meaning = formatted_meaning;
        }

        // Format the other entry's meaning
        let other_formatted = if other.meaning.is_empty() {
            String::new()
        } else {
            format!("*{}*: {}", other.pos, other.meaning)
        };

        // Only add if it's different and not empty
        if !other_formatted.is_empty() && !self.meaning.contains(&other_formatted) {
            if self.meaning.is_empty() {
                self.meaning = other_formatted;
            } else {
                self.meaning = format!("{} | {}", self.meaning, other_formatted);
            }
        }

        // Set POS to Other if we have multiple different parts of speech
        if self.pos != other.pos {
            self.pos = POS::Other;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum POS {
    Noun,
    Verb,
    Adjective,
    Adverb,
    Preposition,
    Conjunction,
    Interjection,
    Article,
    Determiner,
    Pronoun,
    Particle,
    Other,
}

impl POS {
    fn from_str_inner(s: &str) -> Self {
        match s {
            "noun" => POS::Noun,
            "verb" => POS::Verb,
            "adj" => POS::Adjective,
            "article" => POS::Article,
            "adv" => POS::Adverb,
            "prep" => POS::Preposition,
            "conj" => POS::Conjunction,
            "det" => POS::Determiner,
            "pron" => POS::Pronoun,
            "intj" => POS::Interjection,
            "particle" => POS::Particle,
            _ => POS::Other,
        }
    }
}

impl FromStr for POS {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_str_inner(s))
    }
}

impl std::fmt::Display for POS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            POS::Noun => "noun",
            POS::Verb => "verb",
            POS::Adjective => "adjective",
            POS::Adverb => "adverb",
            POS::Preposition => "preposition",
            POS::Conjunction => "conjunction",
            POS::Interjection => "interjection",
            POS::Article => "article",
            POS::Determiner => "determiner",
            POS::Pronoun => "pronoun",
            POS::Particle => "particle",
            POS::Other => "other",
        };
        write!(f, "{}", s)
    }
}

pub fn generate_markdown(glossary: &Glossary) -> String {
    let mut merged_entries: std::collections::HashMap<(String, Language), WordEntry> =
        std::collections::HashMap::new();

    for entry in glossary {
        let key = entry.merge_key();
        match merged_entries.get_mut(&key) {
            Some(existing_entry) => {
                existing_entry.merge_with(entry);
            }
            None => {
                merged_entries.insert(key, entry.clone());
            }
        }
    }

    // Convert back to vector and sort by frequency
    let mut glossary_vec: Vec<WordEntry> = merged_entries.into_values().collect();
    glossary_vec.sort_by(|a, b| b.frequency.cmp(&a.frequency));

    let mut markdown = String::new();
    markdown.push_str("# Glossary\n\n");

    for entry in glossary_vec {
        markdown.push_str(&format!("### {} ({})\n", entry.word, entry.frequency));

        // Handle merged meanings with different POS
        if entry.meaning.contains(" | ") {
            // Multiple POS definitions - create bullet points
            for meaning_part in entry.meaning.split(" | ") {
                if !meaning_part.trim().is_empty() {
                    markdown.push_str(&format!("- {}\n", meaning_part.trim()));
                }
            }
        } else if !entry.meaning.is_empty() {
            // Single definition - check if it already has POS formatting
            if entry.meaning.starts_with('*') && entry.meaning.contains("*:") {
                // Already formatted with POS
                markdown.push_str(&format!("- {}\n", entry.meaning));
            } else {
                // Add POS formatting
                markdown.push_str(&format!("- *{}*: {}\n", entry.pos, entry.meaning));
            }
        }
        markdown.push('\n');
    }

    markdown
}

pub fn write_glossary_to_file(markdown: &str, file_path: &str) -> Result<(), std::io::Error> {
    std::fs::write(file_path, markdown)
}
