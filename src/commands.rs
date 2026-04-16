use futures::{StreamExt, stream::FuturesUnordered};
use std::path::PathBuf;

use crate::{
    Language,
    cli::{Command, FilterAction},
    content::{get_content_from_file, get_word_list_from_content},
    filter::FilterList,
    glossary::{Glossary, WordEntry, generate_markdown, write_glossary_to_file},
    kaikki::get_from_kaikki,
    youtube::get_youtube_transcript,
};

pub async fn handle_command(
    command: Command,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match command {
        Command::Generate {
            file_path,
            lang,
            output,
            filter,
            ai_model,
            ai_url,
            interactive,
            anki,
        } => handle_generate(file_path, lang, output, filter, ai_model, ai_url, interactive, anki).await,
        Command::Youtube {
            video_url,
            lang,
            output,
            filter,
            ai_model,
            ai_url,
            interactive,
            anki,
        } => handle_youtube(video_url, lang, output, filter, ai_model, ai_url, interactive, anki).await,
        Command::Web {
            url,
            lang,
            output,
            filter,
            ai_model,
            ai_url,
            interactive,
            anki,
        } => handle_web(url, lang, output, filter, ai_model, ai_url, interactive, anki).await,
        Command::Filter { action } => handle_filter_action(action).await,
    }
}

async fn handle_generate(
    file_path: String,
    lang: Language,
    output: String,
    filter_file: String,
    ai_model: Option<String>,
    ai_url: String,
    interactive: bool,
    anki: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let file_path = PathBuf::from(file_path);
    if !file_path.exists() {
        return Err("File does not exist".into());
    }

    let content = get_content_from_file(file_path).await?;
    process_content_to_glossary(content, lang, output, filter_file, ai_model, ai_url, interactive, anki).await
}

async fn handle_web(
    url: String,
    lang: Language,
    output: String,
    filter_file: String,
    ai_model: Option<String>,
    ai_url: String,
    interactive: bool,
    anki: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Fetching content from URL: {}...", url);
    let client = reqwest::Client::new();
    let res = client.get(&url).send().await?;
    
    if !res.status().is_success() {
        return Err(format!("Failed to fetch URL: {}", res.status()).into());
    }

    let html = res.text().await?;
    let content = crate::content::extract_text_from_html(&html)?;
    println!("Content fetched successfully!");

    process_content_to_glossary(content, lang, output, filter_file, ai_model, ai_url, interactive, anki).await
}

async fn handle_youtube(
    video_url: String,
    lang: Language,
    output: String,
    filter_file: String,
    ai_model: Option<String>,
    ai_url: String,
    interactive: bool,
    anki: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Fetching transcript from YouTube video...");
    let content = get_youtube_transcript(&video_url, lang).await?;
    println!("Transcript fetched successfully!");

    process_content_to_glossary(content, lang, output, filter_file, ai_model, ai_url, interactive, anki).await
}

async fn process_content_to_glossary(
    content: String,
    lang: Language,
    output: String,
    filter_file: String,
    ai_model: Option<String>,
    ai_url: String,
    interactive: bool,
    anki: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let word_list = get_word_list_from_content(&content);

    // Load filter list and exclude filtered words
    let filter_list = FilterList::load(&filter_file)?;
    let mut filtered_word_list: Vec<(String, (usize, Option<String>))> = word_list
        .into_iter()
        .filter(|(word, _)| !filter_list.contains(word, lang))
        .collect();

    // If interactive mode is enabled, let the user review the words
    if interactive {
        let (kept_words, known_words) = crate::tui::run_tui(filtered_word_list, lang)?;
        
        // Update the filter list with new known words
        if !known_words.is_empty() {
            let mut filter_list = FilterList::load(&filter_file)?;
            for word in known_words {
                filter_list.add(word, lang);
            }
            filter_list.save(&filter_file)?;
            println!("Updated filter list with new known words.");
        }
        
        filtered_word_list = kept_words;
    }

    let mut analysis_results: Vec<(String, usize, Option<String>, Option<String>, Option<String>)> = Vec::new();

    // If AI is enabled, analyze the words
    if let Some(model) = ai_model {
        println!("Analyzing words using AI model '{}'...", model);
        let mut ai_futures = FuturesUnordered::new();

        for (word, (frequency, context)) in filtered_word_list {
            let model = model.clone();
            let ai_url = ai_url.clone();
            let context = context.clone();
            ai_futures.push(tokio::spawn(async move {
                let analysis = if let Some(ctx) = &context {
                    crate::ai::analyze_word(&word, ctx, lang, &model, &ai_url)
                        .await
                        .ok()
                } else {
                    None
                };
                
                let lemma = analysis.as_ref().map(|a| a.lemma.to_lowercase()).unwrap_or(word.clone());
                let cefr = analysis.as_ref().and_then(|a| a.cefr.clone());
                let grammar = analysis.as_ref().and_then(|a| a.grammar.clone());
                
                (lemma, frequency, context, cefr, grammar)
            }));
        }

        while let Some(result) = ai_futures.next().await {
            if let Ok(data) = result {
                analysis_results.push(data);
            }
        }
    } else {
        analysis_results = filtered_word_list.into_iter().map(|(w, (f, c))| (w, f, c, None, None)).collect();
    }

    let mut glossary = Glossary::new();
    let mut futures = FuturesUnordered::new();

    for (word, frequency, context, cefr, grammar) in analysis_results {
        let context = context.clone();
        futures.push(tokio::spawn(async move {
            (word.clone(), frequency, context, cefr, grammar, get_from_kaikki(&word).await)
        }));
    }

    while let Some(result) = futures.next().await {
        match result {
            Ok((_word, frequency, context, cefr, grammar, Ok(entries))) => {
                for entry in entries {
                    if entry.lang_code.to_lowercase() == lang.to_lang_code()
                        && let Some(mut word_entry) =
                            WordEntry::from_kaikki_entry(entry, frequency, context.clone())
                    {
                        word_entry.cefr_level = cefr.clone();
                        word_entry.grammar_note = grammar.clone();
                        glossary.insert(word_entry);
                    }
                }
            }
            Ok((word, _, _, _, _, Err(e))) => eprintln!("Failed to get entry for \"{}\": {}", word, e),
            Err(e) => eprintln!("Task failed: {}", e),
        }
    }

    let mut merged_entries = crate::glossary::get_merged_entries(&glossary);

    // Generate audio if Anki is enabled
    if anki.is_some() {
        println!("Generating audio for Anki deck...");
        let temp_dir = std::env::temp_dir().join("glost_audio");
        let _ = std::fs::remove_dir_all(&temp_dir); // Clear old audio
        std::fs::create_dir_all(&temp_dir)?;
        
        for entry in &mut merged_entries {
            if let Ok(filename) = crate::audio::generate_audio_file(&entry.word, lang, &temp_dir) {
                entry.audio_path = Some(temp_dir.join(filename).to_str().unwrap().to_string());
            }
        }
    }

    if let Some(anki_path) = anki {
        println!("Generating Anki deck in {}...", anki_path);
        crate::anki::generate_anki_deck(&merged_entries, "Glost Deck", &anki_path)?;
    }

    let markdown = generate_markdown(&merged_entries);
    write_glossary_to_file(&markdown, &output)?;

    println!(
        "Generated glossary with {} merged entries in {}",
        merged_entries.len(),
        output
    );
    Ok(())
}

async fn handle_filter_action(
    action: FilterAction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
        FilterAction::Clear { file, lang } => match lang {
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
        },
    }
    Ok(())
}
