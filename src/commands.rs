use futures::{StreamExt, stream::FuturesUnordered};
use std::path::PathBuf;

use crate::{
    Language,
    cli::{Command, FilterAction},
    content::{get_content_from_file, get_expression_list_from_content},
    filter::FilterList,
    glossary::{
        ExpressionEntry, Glossary, generate_markdown, get_merged_entries, write_glossary_to_file,
    },
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
        } => {
            handle_generate(
                file_path,
                lang,
                output,
                filter,
                ai_model,
                ai_url,
                interactive,
                anki,
            )
            .await
        }
        Command::Youtube {
            video_url,
            lang,
            output,
            filter,
            ai_model,
            ai_url,
            interactive,
            anki,
        } => {
            handle_youtube(
                video_url,
                lang,
                output,
                filter,
                ai_model,
                ai_url,
                interactive,
                anki,
            )
            .await
        }
        Command::Web {
            url,
            lang,
            output,
            filter,
            ai_model,
            ai_url,
            interactive,
            anki,
        } => {
            handle_web(
                url,
                lang,
                output,
                filter,
                ai_model,
                ai_url,
                interactive,
                anki,
            )
            .await
        }
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
    process_content_to_glossary(
        content,
        lang,
        output,
        filter_file,
        ai_model,
        ai_url,
        interactive,
        anki,
    )
    .await
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
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()?;
    let res = client.get(&url).send().await?;

    if !res.status().is_success() {
        return Err(format!("Failed to fetch URL: {}", res.status()).into());
    }

    let html = res.text().await?;
    let content = crate::content::extract_text_from_html(&html)?;
    println!("Content fetched successfully!");

    process_content_to_glossary(
        content,
        lang,
        output,
        filter_file,
        ai_model,
        ai_url,
        interactive,
        anki,
    )
    .await
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

    process_content_to_glossary(
        content,
        lang,
        output,
        filter_file,
        ai_model,
        ai_url,
        interactive,
        anki,
    )
    .await
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
    let expression_list = get_expression_list_from_content(&content);
    let cache_path = crate::config::default_cache_file_path();
    let cache = std::sync::Arc::new(crate::cache::Cache::new(&cache_path)?);

    // 1. Initial Filtering
    let filter_list = FilterList::load(&filter_file)?;
    let filtered_list: Vec<(String, (usize, Option<String>))> = expression_list
        .into_iter()
        .filter(|(expr, _)| !filter_list.contains(expr, lang))
        .collect();

    // 2. AI Analysis (Lemmatization, CEFR, Grammar, Grouping)
    let analyzed_expressions: Vec<AnalyzedExpression> = if let Some(model) = ai_model {
        run_ai_analysis(filtered_list, lang, &model, &ai_url, cache.clone()).await?
    } else {
        filtered_list
            .into_iter()
            .map(|(e, (f, c))| (e, f, c, None, None, None))
            .collect()
    };

    // 3. Dictionary Lookup (Kaikki)
    let glossary = fetch_definitions(analyzed_expressions, lang, cache.clone()).await?;
    let mut entries = get_merged_entries(&glossary);

    // 4. Interactive TUI Review (with full analysis)
    if interactive {
        let (kept_entries, known_words) = crate::tui::run_tui(entries, lang)?;

        if !known_words.is_empty() {
            let mut filter_list = FilterList::load(&filter_file)?;
            for word in known_words {
                filter_list.add(word, lang);
            }
            filter_list.save(&filter_file)?;
            println!("Updated filter list with new known expressions.");
        }
        entries = kept_entries;
    }

    // 5. Generate Outputs (Anki, Markdown)
    generate_outputs_from_entries(entries, lang, output, anki).await
}

type AnalyzedExpression = (
    String,
    usize,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);

async fn run_ai_analysis(
    expressions: Vec<(String, (usize, Option<String>))>,
    lang: Language,
    model: &str,
    ai_url: &str,
    cache: std::sync::Arc<crate::cache::Cache>,
) -> Result<Vec<AnalyzedExpression>, Box<dyn std::error::Error + Send + Sync>> {
    let total = expressions.len();
    println!(
        "Analyzing {} expressions using AI model '{}'...",
        total, model
    );

    let pb = indicatif::ProgressBar::new(total as u64);
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )?
            .progress_chars("#>-"),
    );

    let mut ai_futures = FuturesUnordered::new();
    let mut results = Vec::new();

    for (expression, (frequency, context)) in expressions {
        let model = model.to_string();
        let ai_url = ai_url.to_string();
        let context = context.clone();
        let pb = pb.clone();
        let cache = cache.clone();
        ai_futures.push(tokio::spawn(async move {
            let analysis = if let Some(ctx) = &context {
                // Check cache first
                if let Ok(Some(cached)) = cache.get_ai_analysis(&expression, ctx, lang, &model) {
                    Some(cached)
                } else {
                    let result =
                        crate::ai::analyze_expression(&expression, ctx, lang, &model, &ai_url)
                            .await
                            .ok();

                    if let Some(ref analysis) = result {
                        let _ = cache.insert_ai_analysis(&expression, ctx, lang, &model, analysis);
                    }
                    result
                }
            } else {
                None
            };

            pb.inc(1);
            let lemma = analysis
                .as_ref()
                .map(|a| a.lemma.to_lowercase())
                .unwrap_or(expression.clone());
            let meaning = analysis.as_ref().map(|a| a.meaning.clone());
            let cefr = analysis.as_ref().and_then(|a| a.cefr.clone());
            let grammar = analysis.as_ref().and_then(|a| a.grammar.clone());

            (lemma, frequency, context, cefr, grammar, meaning)
        }));
    }

    while let Some(result) = ai_futures.next().await {
        if let Ok(data) = result {
            results.push(data);
        }
    }
    pb.finish_with_message("Analysis complete");
    Ok(results)
}

async fn fetch_definitions(
    analyzed_expressions: Vec<AnalyzedExpression>,
    lang: Language,
    cache: std::sync::Arc<crate::cache::Cache>,
) -> Result<Glossary, Box<dyn std::error::Error + Send + Sync>> {
    let total = analyzed_expressions.len();
    println!("Fetching definitions for {} expressions...", total);

    let pb = indicatif::ProgressBar::new(total as u64);
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.magenta/blue}] {pos}/{len} ({eta})",
            )?
            .progress_chars("#>-"),
    );

    let mut glossary = Glossary::new();
    let mut futures = FuturesUnordered::new();

    for (expr, frequency, context, cefr, grammar, ai_meaning) in analyzed_expressions {
        let context = context.clone();
        let pb = pb.clone();
        let cache = cache.clone();
        futures.push(tokio::spawn(async move {
            // Check cache first
            let result = if let Ok(Some(cached)) = cache.get_kaikki_entries(&expr) {
                Ok(cached)
            } else {
                let res = get_from_kaikki(&expr).await;
                if let Ok(ref entries) = res {
                    let _ = cache.insert_kaikki_entries(&expr, entries);
                }
                res
            };

            pb.inc(1);
            (
                expr.clone(),
                frequency,
                context,
                cefr,
                grammar,
                ai_meaning,
                result,
            )
        }));
    }

    while let Some(result) = futures.next().await {
        match result {
            Ok((_expr, frequency, context, cefr, grammar, ai_meaning, Ok(entries))) => {
                for entry in entries {
                    if entry.lang_code.to_lowercase() == lang.to_lang_code()
                        && let Some(mut expr_entry) = ExpressionEntry::from_kaikki_entry(
                            entry,
                            frequency,
                            context.clone(),
                            grammar.clone(),
                            cefr.clone(),
                        )
                    {
                        if let Some(meaning) = ai_meaning.clone() {
                            expr_entry.meaning =
                                format!("{} (AI: {})", expr_entry.meaning, meaning);
                        }
                        glossary.insert(expr_entry);
                    }
                }
            }
            Ok((expr, _, _, _, _, _, Err(e))) => {
                pb.suspend(|| eprintln!("Failed to get entry for \"{}\": {}", expr, e))
            }
            Err(e) => pb.suspend(|| eprintln!("Task failed: {}", e)),
        }
    }
    pb.finish_with_message("Definitions fetched");
    Ok(glossary)
}

async fn generate_outputs_from_entries(
    mut entries: Vec<ExpressionEntry>,
    lang: Language,
    output_path: String,
    anki_path: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Generate audio if Anki is enabled
    if let Some(ref path) = anki_path {
        println!("Generating audio for Anki deck...");
        let temp_dir = std::env::temp_dir().join("glost_audio");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir)?;

        for entry in &mut entries {
            if let Ok(filename) =
                crate::audio::generate_audio_file(&entry.expression, lang, &temp_dir)
            {
                entry.audio_path = Some(temp_dir.join(filename).to_str().unwrap().to_string());
            }
        }

        println!("Generating Anki deck in {}...", path);
        crate::anki::generate_anki_deck(&entries, "Glost Deck", path)?;
    }

    let markdown = generate_markdown(&entries);
    write_glossary_to_file(&markdown, &output_path)?;

    println!(
        "Generated glossary with {} merged entries in {}",
        entries.len(),
        output_path
    );
    Ok(())
}

async fn handle_filter_action(
    action: FilterAction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match action {
        FilterAction::Add {
            words: expressions,
            file,
            lang,
        } => {
            let mut filter_list = FilterList::load(&file)?;
            for expr in expressions {
                filter_list.add(expr.clone(), lang);
                println!("Added '{}' to {} filter list", expr, lang);
            }
            filter_list.save(&file)?;
        }
        FilterAction::Remove {
            words: expressions,
            file,
            lang,
        } => {
            let mut filter_list = FilterList::load(&file)?;
            for expr in expressions {
                if filter_list.remove(&expr, lang) {
                    println!("Removed '{}' from {} filter list", expr, lang);
                } else {
                    println!("Expression '{}' was not in {} filter list", expr, lang);
                }
            }
            filter_list.save(&file)?;
        }
        FilterAction::List { file, lang } => {
            let filter_list = FilterList::load(&file)?;
            let entries = filter_list.list(lang);
            if entries.is_empty() {
                match lang {
                    Some(l) => println!("Filter list for {} is empty", l),
                    None => println!("Filter list is empty"),
                }
            } else {
                match lang {
                    Some(l) => {
                        println!("Filter list for {} contains {} entries:", l, entries.len())
                    }
                    None => println!("Filter list contains {} entries:", entries.len()),
                }
                let mut current_lang: Option<Language> = None;
                for (expr_lang, expr) in entries {
                    if current_lang != Some(expr_lang) {
                        if lang.is_none() {
                            println!("\n{}:", expr_lang);
                        }
                        current_lang = Some(expr_lang);
                    }
                    println!("  {}", expr);
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
