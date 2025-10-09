use futures::{stream::FuturesUnordered, StreamExt};
use std::path::PathBuf;

use crate::{
    cli::{Command, FilterAction},
    content::{get_content_from_file, get_word_list_from_content},
    filter::FilterList,
    glossary::{generate_markdown, write_glossary_to_file, Glossary, WordEntry},
    kaikki::get_from_kaikki,
    youtube::get_youtube_transcript,
    Language,
};

pub async fn handle_command(command: Command) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Command::Generate { file_path, lang, output, filter } => {
            handle_generate(file_path, lang, output, filter).await
        }
        Command::Youtube { video_url, lang, output, filter } => {
            handle_youtube(video_url, lang, output, filter).await
        }
        Command::Filter { action } => {
            handle_filter_action(action).await
        }
    }
}

async fn handle_generate(
    file_path: String,
    lang: Language,
    output: String,
    filter_file: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = PathBuf::from(file_path);
    if !file_path.exists() {
        return Err("File does not exist".into());
    }

    let content = get_content_from_file(file_path)?;
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

    println!("Generated glossary with {} entries in {}", glossary.len(), output);
    Ok(())
}

async fn handle_youtube(
    video_url: String,
    lang: Language,
    output: String,
    filter_file: String,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Fetching transcript from YouTube video...");
    let content = get_youtube_transcript(&video_url).await?;
    println!("Transcript fetched successfully!");
    
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

    println!("Generated glossary with {} entries in {}", glossary.len(), output);
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