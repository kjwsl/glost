pub mod kaikki;
use std::{collections::HashSet, fs, path::Path, str::FromStr};

use epub::doc::EpubDoc;

pub type Glossary = HashSet<WordEntry>;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WordEntry {
    pub word: String,
    pub meaning: String,
    pub pos: POS,
    pub lang: Language,
}

impl WordEntry {
    pub fn new(word: String, meaning: String, pos: POS, lang: Language) -> Self {
        Self {
            word,
            meaning,
            pos,
            lang,
        }
    }

    pub fn from_kaikki_entry(entry: kaikki::Entry) -> Option<Self> {
        let pos = match entry.pos.as_str() {
            "noun" => POS::Noun,
            "verb" => POS::Verb,
            "adj" => POS::Adjective,
            "adv" => POS::Adverb,
            "prep" => POS::Preposition,
            "conj" => POS::Conjunction,
            "interj" => POS::Interjection,
            _ => POS::Other,
        };

        let lang = match Language::from_str(&entry.lang) {
            Ok(l) => l,
            Err(_) => return None,
        };

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
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Language {
    Afrikaans,
    Chinese,
    Dutch,
    English,
    French,
    German,
    Italian,
    Japanese,
    Korean,
    Mandarin,
    Portuguese,
    Russian,
    Spanish,
}

impl Language {
    pub fn to_lang_code(&self) -> &str {
        match self {
            Language::Afrikaans => "af",
            Language::Chinese => "zh",
            Language::Dutch => "nl",
            Language::English => "en",
            Language::French => "fr",
            Language::German => "de",
            Language::Italian => "it",
            Language::Japanese => "ja",
            Language::Korean => "ko",
            Language::Mandarin => "zh",
            Language::Portuguese => "pt",
            Language::Russian => "ru",
            Language::Spanish => "es",
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Language::Afrikaans => "Afrikaans",
            Language::Chinese => "Chinese",
            Language::Dutch => "Dutch",
            Language::English => "English",
            Language::French => "French",
            Language::German => "German",
            Language::Italian => "Italian",
            Language::Japanese => "Japanese",
            Language::Korean => "Korean",
            Language::Mandarin => "Mandarin",
            Language::Portuguese => "Portuguese",
            Language::Russian => "Russian",
            Language::Spanish => "Spanish",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for Language {
    type Err = std::fmt::Error;
    fn from_str(s: &str) -> Result<Self, std::fmt::Error> {
        match s.to_lowercase().as_str() {
            "afrikaans" => Ok(Language::Afrikaans),
            "chinese" => Ok(Language::Chinese),
            "dutch" => Ok(Language::Dutch),
            "english" => Ok(Language::English),
            "french" => Ok(Language::French),
            "german" => Ok(Language::German),
            "italian" => Ok(Language::Italian),
            "japanese" => Ok(Language::Japanese),
            "korean" => Ok(Language::Korean),
            "mandarin" => Ok(Language::Mandarin),
            "portuguese" => Ok(Language::Portuguese),
            "russian" => Ok(Language::Russian),
            "spanish" => Ok(Language::Spanish),
            _ => Err(std::fmt::Error),
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
    Other,
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
            POS::Other => "other",
        };
        write!(f, "{}", s)
    }
}

pub async fn get_from_kaikki(
    word: &str,
) -> Result<Vec<kaikki::Entry>, Box<dyn std::error::Error + Send + Sync>> {
    if word.is_empty() {
        return Err("Word is empty".into());
    }
    let lower_word = word.to_lowercase();
    let ch1 = lower_word.chars().next().unwrap();
    let ch2_opt = lower_word.chars().nth(1);

    let part2 = match ch2_opt {
        Some(ch2) => format!("{ch1}{ch2}"),
        None => format!("{ch1}"),
    };

    let url = format!(
        "https://kaikki.org/dictionary/All%20languages%20combined/meaning/{ch1}/{part2}/{lower_word}.jsonl"
    );

    let mut retries = 3;
    let mut delay = std::time::Duration::from_millis(100);

    let resp = loop {
        match reqwest::get(&url).await {
            Ok(resp) => break resp,
            Err(e) => {
                if retries == 0 {
                    return Err(e.into());
                }
                retries -= 1;
                tokio::time::sleep(delay).await;
                delay *= 2;
            }
        }
    };

    if !resp.status().is_success() {
        return Err(format!("Failed to get entry from kaikki.org: {}", resp.status()).into());
    }

    let text = resp.text().await?;
    let mut entries = vec![];

    for line in text.lines() {
        if line.is_empty() {
            continue;
        }
        let entry = match serde_json::from_str::<kaikki::Entry>(line) {
            Ok(e) => e,
            Err(e) => return Err(format!("Failed to parse entry: {e}").into()),
        };
        entries.push(entry);
    }

    Ok(entries)
}

pub fn get_content_from_epub(
    file_path: impl AsRef<Path>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut doc = EpubDoc::new(file_path)?;
    let mut content = String::new();

    while let Some((chapter_content, _mime_type)) = doc.get_current_str() {
        content.push_str(&chapter_content);
        if !doc.go_next() {
            break;
        }
    }

    Ok(content)
}

pub fn get_content_from_pdf(
    file_path: impl AsRef<Path>,
) -> Result<String, Box<dyn std::error::Error>> {
    let bytes = fs::read(file_path)?;
    let out = pdf_extract::extract_text_from_mem(bytes.as_slice())?;

    Ok(out)
}

pub fn get_word_list_from_content(text: &str) -> HashSet<String> {
    text.split_whitespace()
        .map(String::from)
        .filter(|s| is_word(s))
        .collect()
}

fn is_word(word: &str) -> bool {
    word.chars().all(char::is_alphabetic)
}
