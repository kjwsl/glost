use std::path::PathBuf;

use clap::Parser;
use glossary::{get_content_from_epub, get_content_from_pdf, get_from_kaikki, get_word_list_from_content, Glossary, WordEntry};

#[derive(Debug, Clone, Parser)]
struct Args {
    file_path: String,
    #[clap(short, long, default_value = "en")]
    lang: String,
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

    for word in word_list {
        let entries = get_from_kaikki(&word).await?;
        for entry in entries {
            if entry.lang.to_lowercase() == args.lang {
                if let Some(word_entry) = WordEntry::from_kaikki_entry(entry) {
                    glossary.insert(word_entry);
                }
            }
        }
    }


    Ok(())
}
