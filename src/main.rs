use futures::{stream::FuturesUnordered, StreamExt};
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use glossary::{
    get_content_from_epub, get_content_from_pdf, get_from_kaikki, get_word_list_from_content, 
    Glossary, Language, WordEntry, FilterList
};

#[derive(Debug, Clone, Parser)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Clone, Subcommand)]
enum Command {
    /// Generate a glossary from a file
    Generate {
        file_path: String,
        #[clap(short, long, default_value_t = Language::English)]
        lang: Language,
        #[clap(short, long, default_value = "glossary.md")]
        output: String,
        #[clap(short, long, default_value = "filter.txt")]
        filter: String,
    },
    /// Manage filter list of known words
    Filter {
        #[clap(subcommand)]
        action: FilterAction,
    },
}

#[derive(Debug, Clone, Subcommand)]
enum FilterAction {
    /// Add words to the filter list
    Add {
        words: Vec<String>,
        #[clap(short, long, default_value = "filter.txt")]
        file: String,
        #[clap(short, long, default_value_t = Language::English)]
        lang: Language,
    },
    /// Remove words from the filter list
    Remove {
        words: Vec<String>,
        #[clap(short, long, default_value = "filter.txt")]
        file: String,
        #[clap(short, long, default_value_t = Language::English)]
        lang: Language,
    },
    /// List all words in the filter list
    List {
        #[clap(short, long, default_value = "filter.txt")]
        file: String,
        #[clap(short, long)]
        lang: Option<Language>,
    },
    /// Clear words from the filter list
    Clear {
        #[clap(short, long, default_value = "filter.txt")]
        file: String,
        #[clap(short, long)]
        lang: Option<Language>,
    },
}

fn generate_markdown(glossary: &Glossary) -> String {
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

fn write_glossary_to_file(markdown: &str, file_path: &str) -> Result<(), std::io::Error> {
    std::fs::write(file_path, markdown)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.command {
        Command::Generate { file_path, lang, output, filter } => {
            generate_glossary(file_path, lang, output, filter).await
        }
        Command::Filter { action } => {
            handle_filter_action(action).await
        }
    }
}

async fn generate_glossary(
    file_path: String,
    lang: Language,
    output: String,
    filter_file: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = PathBuf::from(file_path);
    if !file_path.exists() {
        return Err("File does not exist".into());
    }
    let ext = file_path
        .extension()
        .ok_or("File has no extension")?
        .to_str()
        .unwrap();
    let content = match ext {
        "epub" => get_content_from_epub(file_path)?,
        "pdf" => get_content_from_pdf(file_path)?,
        "txt" => std::fs::read_to_string(file_path)?,
        _ => {
            return Err("Unsupported file extension".into());
        }
    };

    let word_list = get_word_list_from_content(&content);

    // Load filter list and exclude filtered words
    let filter_list = FilterList::load(&filter_file)?;
    let filtered_word_list: Vec<(String, usize)> = word_list
        .into_iter()
        .filter(|(word, _)| !filter_list.contains(word, lang))
        .collect();

    let mut glossary = Glossary::new();
    let mut futures = FuturesUnordered::new();

    for (word, frequency) in filtered_word_list {
        futures.push(tokio::spawn(async move {
            (word.clone(), frequency, get_from_kaikki(&word).await)
        }));
    }

    while let Some(result) = futures.next().await {
        match result {
            Ok((_word, frequency, Ok(entries))) => {
                for entry in entries {
                    if entry.lang_code.to_lowercase() == lang.to_lang_code() {
                        if let Some(word_entry) = WordEntry::from_kaikki_entry(entry, frequency) {
                            glossary.insert(word_entry);
                        }
                    }
                }
            }
            Ok((word, _, Err(e))) => eprintln!("Failed to get entry for \"{}\": {}", word, e),
            Err(e) => eprintln!("Task failed: {}", e),
        }
    }

    let markdown = generate_markdown(&glossary);
    write_glossary_to_file(&markdown, &output)?;

    Ok(())
}

async fn handle_filter_action(action: FilterAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        FilterAction::Add { words, file, lang } => {
            let mut filter_list = FilterList::load(&file)?;
            for word in words {
                filter_list.add(word.clone(), lang);
                println!("Added '{}' to {} filter list", word, lang);
            }
            filter_list.save(&file)?;
        }
        FilterAction::Remove { words, file, lang } => {
            let mut filter_list = FilterList::load(&file)?;
            for word in words {
                if filter_list.remove(&word, lang) {
                    println!("Removed '{}' from {} filter list", word, lang);
                } else {
                    println!("Word '{}' was not in {} filter list", word, lang);
                }
            }
            filter_list.save(&file)?;
        }
        FilterAction::List { file, lang } => {
            let filter_list = FilterList::load(&file)?;
            let words = filter_list.list(lang);
            if words.is_empty() {
                match lang {
                    Some(l) => println!("Filter list for {} is empty", l),
                    None => println!("Filter list is empty"),
                }
            } else {
                match lang {
                    Some(l) => println!("Filter list for {} contains {} words:", l, words.len()),
                    None => println!("Filter list contains {} words:", words.len()),
                }
                let mut current_lang: Option<Language> = None;
                for (word_lang, word) in words {
                    if current_lang != Some(word_lang) {
                        if lang.is_none() {
                            println!("\n{}:", word_lang);
                        }
                        current_lang = Some(word_lang);
                    }
                    println!("  {}", word);
                }
            }
        }
        FilterAction::Clear { file, lang } => {
            match lang {
                Some(l) => {
                    let mut filter_list = FilterList::load(&file)?;
                    filter_list.clear_language(l);
                    filter_list.save(&file)?;
                    println!("Cleared {} filter list", l);
                }
                None => {
                    let filter_list = FilterList::new();
                    filter_list.save(&file)?;
                    println!("Cleared all filter lists");
                }
            }
        }
    }
    Ok(())
}
