use std::{collections::HashSet, str::FromStr};
use crate::{kaikki, Language};

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
        let pos = POS::from_str(&entry.pos);
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
    pub fn from_str(s: &str) -> Self {
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
    let mut glossary_vec = glossary.iter().collect::<Vec<_>>();
    glossary_vec.sort_by(|a, b| b.frequency.cmp(&a.frequency));

    let mut markdown = String::new();
    markdown.push_str("# Glossary\n\n");

    for entry in glossary_vec {
        markdown.push_str(&format!("### {} ({})\n", entry.word, entry.frequency));
        markdown.push_str(&format!("- *{}*: {}\n\n", entry.pos, entry.meaning));
    }

    markdown
}

pub fn write_glossary_to_file(markdown: &str, file_path: &str) -> Result<(), std::io::Error> {
    std::fs::write(file_path, markdown)
}