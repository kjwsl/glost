pub mod cli;
pub mod commands;
pub mod config;
pub mod content;
pub mod filter;
pub mod flashcards;
pub mod glossary;
pub mod kaikki;
pub mod language;
pub mod server;
pub mod youtube;

pub use filter::FilterList;
pub use flashcards::{Flashcard, FlashcardList};
pub use glossary::{Glossary, POS, WordEntry};
pub use language::Language;
pub use server::start_server;

// Re-export for backward compatibility and convenience
pub use content::{get_content_from_epub, get_content_from_pdf, get_word_list_from_content};
pub use kaikki::get_from_kaikki;
pub use youtube::get_youtube_transcript;
