use futures::{StreamExt, stream::FuturesUnordered};
use std::path::PathBuf;
use std::sync::Arc;

use crate::{
    Language,
    ai::OllamaLinguist,
    cache::Cache,
    cli::{Command, FilterAction},
    content::{get_content_from_file, get_expression_list_from_content},
    filter::FilterList,
    glossary::{
        ExpressionEntry, Glossary, generate_markdown, get_merged_entries, write_glossary_to_file,
    },
    kaikki::get_from_kaikki,
    youtube::get_youtube_transcript,
};

// 1. Group the repetitive parameters into a configuration struct.
pub struct PipelineConfig {
    pub lang: Language,
    pub output: String,
    pub filter_file: String,
    pub interactive: bool,
    pub anki: Option<String>,
}

// 2. The pipeline owns the configuration, the AI client, and the cache.
pub struct GlossaryPipeline {
    config: PipelineConfig,
    linguist: Option<Arc<OllamaLinguist>>,
    cache: Arc<Cache>,
    http_client: reqwest::Client,
}

impl GlossaryPipeline {
    pub fn new(
        config: PipelineConfig,
        ai_model: Option<String>,
        ai_url: String,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let cache_path = crate::config::default_cache_file_path();
        let cache = Arc::new(Cache::new(&cache_path)?);

        let linguist = ai_model.map(|model| Arc::new(OllamaLinguist::new(ai_url, model)));

        let http_client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()?;

        Ok(Self {
            config,
            linguist,
            cache,
            http_client,
        })
    }

    pub async fn process_file(
        &self,
        file_path: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let path = PathBuf::from(file_path);
        if !path.exists() {
            return Err("File does not exist".into());
        }
        let content = get_content_from_file(path).await?;
        self.process_content(content).await
    }

    pub async fn process_web(
        &self,
        url: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("Fetching content from URL: {}...", url);
        let res = self.http_client.get(&url).send().await?;

        if !res.status().is_success() {
            return Err(format!("Failed to fetch URL: {}", res.status()).into());
        }

        let html = res.text().await?;
        let content = crate::content::extract_text_from_html(&html)?;
        println!("Content fetched successfully!");

        self.process_content(content).await
    }

    pub async fn process_youtube(
        &self,
        video_url: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("Fetching transcript from YouTube video...");
        // Assuming get_youtube_transcript can be adapted to take &self.http_client if needed
        let content = get_youtube_transcript(&video_url, self.config.lang).await?;
        println!("Transcript fetched successfully!");

        self.process_content(content).await
    }

    async fn process_content(
        &self,
        content: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let analyzed_expressions = if let Some(linguist) = &self.linguist {
            self.run_ai_lemmatization(content, linguist).await?
        } else {
            let expression_list = get_expression_list_from_content(&content);
            let filter_list = FilterList::load(&self.config.filter_file)?;
            expression_list
                .into_iter()
                .filter(|(expr, _)| !filter_list.contains(expr, self.config.lang))
                .map(|(e, (f, c))| AnalyzedExpression {
                    original: e.clone(),
                    lemma: e,
                    frequency: f,
                    context: c,
                    cefr: None,
                    grammar: None,
                    meaning: None,
                })
                .collect()
        };

        // Dictionary Lookup
        let glossary = self.fetch_definitions(analyzed_expressions).await?;
        let mut entries = get_merged_entries(&glossary);

        // Interactive Review
        if self.config.interactive {
            let (kept_entries, known_words) =
                crate::tui::run_tui(entries.as_slice(), self.config.lang)?;

            if !known_words.is_empty() {
                let mut filter_list = FilterList::load(&self.config.filter_file)?;
                for word in known_words {
                    filter_list.add(word, self.config.lang);
                }
                filter_list.save(&self.config.filter_file)?;
                println!("Updated filter list with new known expressions.");
            }
            entries = kept_entries;
        }

        // 5. Output
        self.generate_outputs(entries).await
    }

    #[allow(dead_code)]
    async fn run_ai_analysis(
        &self,
        expressions: Vec<(String, (usize, Option<String>))>,
        linguist: &Arc<OllamaLinguist>,
    ) -> Result<Vec<AnalyzedExpression>, Box<dyn std::error::Error + Send + Sync>> {
        let total = expressions.len();
        println!("Analyzing {} expressions using AI...", total);

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
            let context = context.clone();
            let pb = pb.clone();
            let cache = self.cache.clone();
            let linguist = linguist.clone();
            let lang = self.config.lang;

            ai_futures.push(tokio::spawn(async move {
                let analysis = if let Some(ctx) = &context {
                    if let Ok(Some(cached)) =
                        cache.get_ai_analysis(&expression, ctx, lang, linguist.model_name())
                    {
                        Some(cached)
                    } else {
                        // Notice we now use the struct's method
                        let result = linguist
                            .analyze_expression(&expression, ctx, lang)
                            .await
                            .ok();
                        if let Some(ref analysis) = result {
                            let _ = cache.insert_ai_analysis(
                                &expression,
                                ctx,
                                lang,
                                linguist.model_name(),
                                analysis,
                            );
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
                    .unwrap_or_else(|| expression.clone());

                AnalyzedExpression {
                    original: expression,
                    lemma,
                    frequency,
                    context,
                    cefr: analysis.as_ref().and_then(|a| a.cefr.clone()),
                    grammar: analysis.as_ref().and_then(|a| a.grammar.clone()),
                    meaning: analysis.as_ref().map(|a| a.meaning.clone()),
                }
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

    async fn run_ai_lemmatization(
        &self,
        content: String,
        linguist: &Arc<OllamaLinguist>,
    ) -> Result<Vec<AnalyzedExpression>, Box<dyn std::error::Error + Send + Sync>> {
        use unicode_segmentation::UnicodeSegmentation;

        let sentences: Vec<String> = content
            .unicode_sentences()
            .map(|s| s.trim().replace('\n', " "))
            .filter(|s| !s.is_empty())
            .collect();

        if sentences.is_empty() {
            return Ok(vec![]);
        }

        println!("Lemmatizing content using AI...");

        let mut all_expressions: std::collections::HashMap<String, (usize, Option<String>)> =
            std::collections::HashMap::new();

        let pb = indicatif::ProgressBar::new(sentences.len() as u64);
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
                )?
                .progress_chars("#>-"),
        );

        let mut futures = FuturesUnordered::new();

        for sentence in sentences {
            let pb = pb.clone();
            let linguist = linguist.clone();
            let lang = self.config.lang;

            futures.push(tokio::spawn(async move {
                let lemmas = linguist.lemmatize_sentence(&sentence, lang).await;
                pb.inc(1);
                (sentence, lemmas)
            }));
        }

        while let Some(result) = futures.next().await {
            match result {
                Ok((sentence, Ok(lemmas))) => {
                    for lemma in lemmas {
                        if lemma.is_empty() || lemma == "null" {
                            continue;
                        }
                        let entry = all_expressions
                            .entry(lemma.to_lowercase())
                            .or_insert((0, None));
                        entry.0 += 1;
                        if entry.1.is_none() {
                            entry.1 = Some(sentence.clone());
                        }
                    }
                }
                Ok((_, Err(e))) => {
                    eprintln!("Lemmatization error: {:?}", e);
                }
                Err(e) => {
                    eprintln!("Task error: {:?}", e);
                }
            }
        }

        pb.finish_with_message("Lemmatization complete");

        let filter_list = FilterList::load(&self.config.filter_file)?;

        let analyzed: Vec<AnalyzedExpression> = all_expressions
            .into_iter()
            .filter(|(expr, _)| !filter_list.contains(expr, self.config.lang))
            .map(|(lemma, (frequency, context))| AnalyzedExpression {
                original: lemma.clone(),
                lemma,
                frequency,
                context,
                cefr: None,
                grammar: None,
                meaning: None,
            })
            .collect();

        println!(
            "Found {} unique expressions after filtering",
            analyzed.len()
        );
        Ok(analyzed)
    }

    async fn fetch_definitions(
        &self,
        analyzed_expressions: Vec<AnalyzedExpression>,
    ) -> Result<Glossary, Box<dyn std::error::Error + Send + Sync>> {
        let total = analyzed_expressions.len();
        println!("Fetching definitions for {} expressions...", total);

        let pb = indicatif::ProgressBar::new(total as u64);
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.magenta/blue}] {pos}/{len} ({eta})")?
                .progress_chars("#>-"),
        );

        let mut glossary = Glossary::new();
        let mut futures = FuturesUnordered::new();

        for expr in analyzed_expressions {
            let pb = pb.clone();
            let cache = self.cache.clone();

            futures.push(tokio::spawn(async move {
                let result = if let Ok(Some(cached)) = cache.get_kaikki_entries(&expr.lemma) {
                    Ok(cached)
                } else {
                    let res = get_from_kaikki(&expr.lemma).await;
                    if let Ok(ref entries) = res {
                        let _ = cache.insert_kaikki_entries(&expr.lemma, entries);
                    }
                    res
                };
                pb.inc(1);
                (expr, result)
            }));
        }

        while let Some(result) = futures.next().await {
            match result {
                Ok((expr, Ok(entries))) => {
                    let target_lang = self.config.lang.to_lang_code();
                    let matching_entries: Vec<_> = entries
                        .into_iter()
                        .filter(|e| e.lang_code.to_lowercase() == target_lang)
                        .collect();

                    if matching_entries.is_empty() && expr.original != expr.lemma {
                        if let Ok(fallback_entries) = get_from_kaikki(&expr.original).await {
                            for entry in fallback_entries {
                                if entry.lang_code.to_lowercase() == target_lang
                                    && let Some(mut expr_entry) = ExpressionEntry::from_kaikki_entry(
                                        entry,
                                        expr.frequency,
                                        expr.context.clone(),
                                        expr.grammar.clone(),
                                        expr.cefr.clone(),
                                    )
                                {
                                    if let Some(meaning) = &expr.meaning {
                                        expr_entry.meaning =
                                            format!("{} (AI: {})", expr_entry.meaning, meaning);
                                    }
                                    glossary.insert(expr_entry);
                                }
                            }
                        }
                    } else {
                        for entry in matching_entries {
                            if let Some(mut expr_entry) = ExpressionEntry::from_kaikki_entry(
                                entry,
                                expr.frequency,
                                expr.context.clone(),
                                expr.grammar.clone(),
                                expr.cefr.clone(),
                            ) {
                                if let Some(meaning) = &expr.meaning {
                                    expr_entry.meaning =
                                        format!("{} (AI: {})", expr_entry.meaning, meaning);
                                }
                                glossary.insert(expr_entry);
                            }
                        }
                    }
                }
                Ok((expr, Err(e))) => {
                    if expr.original != expr.lemma {
                        if let Ok(entries) = get_from_kaikki(&expr.original).await {
                            for entry in entries {
                                if entry.lang_code.to_lowercase() == self.config.lang.to_lang_code()
                                {
                                    if let Some(mut expr_entry) = ExpressionEntry::from_kaikki_entry(
                                        entry,
                                        expr.frequency,
                                        expr.context.clone(),
                                        expr.grammar.clone(),
                                        expr.cefr.clone(),
                                    ) {
                                        if let Some(meaning) = &expr.meaning {
                                            expr_entry.meaning =
                                                format!("{} (AI: {})", expr_entry.meaning, meaning);
                                        }
                                        glossary.insert(expr_entry);
                                    }
                                }
                            }
                        }
                    } else {
                        pb.suspend(|| {
                            eprintln!("Failed to get entry for \"{}\": {}", expr.original, e)
                        })
                    }
                }
                Err(e) => pb.suspend(|| eprintln!("Task failed: {}", e)),
            }
        }
        pb.finish_with_message("Definitions fetched");
        Ok(glossary)
    }

    async fn generate_outputs(
        &self,
        mut entries: Vec<ExpressionEntry>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(ref path) = self.config.anki {
            println!("Generating audio for Anki deck...");
            let temp_dir = std::env::temp_dir().join("glost_audio");
            let _ = std::fs::remove_dir_all(&temp_dir);
            std::fs::create_dir_all(&temp_dir)?;

            for entry in &mut entries {
                if let Ok(filename) = crate::audio::generate_audio_file(
                    &entry.expression,
                    self.config.lang,
                    &temp_dir,
                ) {
                    entry.audio_path = Some(temp_dir.join(filename).to_str().unwrap().to_string());
                }
            }

            println!("Generating Anki deck in {}...", path);
            crate::anki::generate_anki_deck(&entries, "Glost Deck", path)?;
        }

        let markdown = generate_markdown(&entries);
        write_glossary_to_file(&markdown, &self.config.output)?;

        println!(
            "Generated glossary with {} merged entries in {}",
            entries.len(),
            self.config.output
        );
        Ok(())
    }
}

pub struct AnalyzedExpression {
    original: String,
    lemma: String,
    frequency: usize,
    context: Option<String>,
    cefr: Option<String>,
    grammar: Option<String>,
    meaning: Option<String>,
}

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
            let config = PipelineConfig {
                lang,
                output,
                filter_file: filter,
                interactive,
                anki,
            };
            let pipeline = GlossaryPipeline::new(config, ai_model, ai_url)?;
            pipeline.process_file(file_path).await
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
            let config = PipelineConfig {
                lang,
                output,
                filter_file: filter,
                interactive,
                anki,
            };
            let pipeline = GlossaryPipeline::new(config, ai_model, ai_url)?;
            pipeline.process_youtube(video_url).await
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
            let config = PipelineConfig {
                lang,
                output,
                filter_file: filter,
                interactive,
                anki,
            };
            let pipeline = GlossaryPipeline::new(config, ai_model, ai_url)?;
            pipeline.process_web(url).await
        }
        Command::Filter { action } => handle_filter_action(action).await,
    }
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
