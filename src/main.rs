use futures::{stream::FuturesUnordered, StreamExt};
use std::path::PathBuf;

use clap::Parser;
use glossary::{
    get_content_from_epub, get_content_from_pdf, get_from_kaikki, get_word_list_from_content, Glossary, Language, WordEntry
};

#[derive(Debug, Clone, Parser)]
struct Args {
    file_path: String,
    #[clap(short, long, default_value_t = Language::English)]
    lang: Language,
    #[clap(short, long, default_value = "glossary.md")]
    output: String,
}

fn generate_markdown(glossary: &Glossary) -> String {
    let mut glossary_vec = glossary.into_iter().collect::<Vec<_>>();
    glossary_vec.sort();

    let mut grouped_glossary = std::collections::BTreeMap::new();
    for entry in glossary_vec {
        let first_char = entry.word.chars().next().unwrap().to_ascii_uppercase();
        grouped_glossary.entry(first_char).or_insert(vec![]).push(entry.clone());
    }

    let mut markdown = String::new();
    markdown.push_str("# Glossary\n\n");

    for (char, entries) in grouped_glossary {
        markdown.push_str(&format!("## {}\n\n", char));
        for entry in entries {
            markdown.push_str(&format!("### {}\n", entry.word));
            markdown.push_str(&format!("- *{}*: {}\n\n", entry.pos.to_string(), entry.meaning));
        }
    }

    markdown
}

fn write_glossary_to_file(markdown: &str, file_path: &str) -> Result<(), std::io::Error> {
    std::fs::write(file_path, markdown)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let file_path = PathBuf::from(args.file_path);
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
        _ => {
            return Err("Unsupported file extension".into());
        }
    };

    let word_list = get_word_list_from_content(&content);

    let mut glossary = Glossary::new();

    let mut futures = FuturesUnordered::new();

    for word in word_list {
        futures.push(tokio::spawn(async move {
            (word.clone(), get_from_kaikki(&word).await)
        }));
    }

    while let Some(result) = futures.next().await {
        match result {
            Ok((_word, Ok(entries))) => {
                for entry in entries {
                    if entry.lang_code.to_lowercase() == args.lang.to_lang_code() {
                        if let Some(word_entry) = WordEntry::from_kaikki_entry(entry) {
                            glossary.insert(word_entry);
                        }
                    }
                }
            }
            Ok((word, Err(e))) => eprintln!("Failed to get entry for \"{}\": {}", word, e),
            Err(e) => eprintln!("Task failed: {}", e),
        }
    }

    let markdown = generate_markdown(&glossary);
    write_glossary_to_file(&markdown, &args.output)?;

    Ok(())
}
