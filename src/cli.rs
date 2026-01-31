use clap::{Parser, Subcommand};
use crate::{Language, config::default_filter_file_path};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// Generate a glossary from a file
    Generate {
        file_path: String,
        #[clap(short, long, default_value_t = Language::English)]
        lang: Language,
        #[clap(short, long, default_value = "glossary.md")]
        output: String,
        #[clap(short, long, default_value_t = default_filter_file_path())]
        filter: String,
    },
    /// Generate a glossary from a YouTube video transcript
    Youtube {
        video_url: String,
        #[clap(short, long, default_value_t = Language::English)]
        lang: Language,
        #[clap(short, long, default_value = "glossary.md")]
        output: String,
        #[clap(short, long, default_value_t = default_filter_file_path())]
        filter: String,
    },
    /// Serve the interactive reader
    Serve {
        #[clap(short, long, default_value_t = 3000)]
        port: u16,
        #[clap(short, long, default_value = ".")]
        dir: String,
    },
    /// Manage filter list of known words
    Filter {
        #[clap(subcommand)]
        action: FilterAction,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum FilterAction {
    /// Add words to the filter list
    Add {
        words: Vec<String>,
        #[clap(short, long, default_value_t = default_filter_file_path())]
        file: String,
        #[clap(short, long, default_value_t = Language::English)]
        lang: Language,
    },
    /// Remove words from the filter list
    Remove {
        words: Vec<String>,
        #[clap(short, long, default_value_t = default_filter_file_path())]
        file: String,
        #[clap(short, long, default_value_t = Language::English)]
        lang: Language,
    },
    /// List all words in the filter list
    List {
        #[clap(short, long, default_value_t = default_filter_file_path())]
        file: String,
        #[clap(short, long)]
        lang: Option<Language>,
    },
    /// Clear words from the filter list
    Clear {
        #[clap(short, long, default_value_t = default_filter_file_path())]
        file: String,
        #[clap(short, long)]
        lang: Option<Language>,
    },
}