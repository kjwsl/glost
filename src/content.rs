use aho_corasick::AhoCorasick;
use epub::doc::EpubDoc;
use scraper::{Html, Selector};
use std::sync::LazyLock;
use std::{collections::HashMap, path::Path};

use crate::youtube::extract_text_from_vtt;

pub async fn get_content_from_file(
    file_path: impl AsRef<Path>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let file_path = file_path.as_ref();
    let ext = file_path
        .extension()
        .ok_or("File has no extension")?
        .to_str()
        .unwrap();

    match ext {
        "epub" => get_content_from_epub(file_path).await,
        "pdf" => get_content_from_pdf(file_path).await,
        "txt" => Ok(tokio::fs::read_to_string(file_path).await?),
        "vtt" => extract_text_from_vtt(&tokio::fs::read_to_string(file_path).await?),
        "srt" => extract_text_from_srt(&tokio::fs::read_to_string(file_path).await?),
        _ => Err(format!("Unsupported file extension: {}", ext).into()),
    }
}

pub async fn get_content_from_epub(
    file_path: impl AsRef<Path>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let file_path = file_path.as_ref().to_path_buf();

    // epub::doc::EpubDoc blocks and does synchronous zip/file parsing
    let out = tokio::task::spawn_blocking(move || {
        let mut doc = EpubDoc::new(file_path)?;
        let mut content = String::new();

        while let Some((chapter_content, _mime_type)) = doc.get_current_str() {
            content.push_str(&chapter_content);
            if !doc.go_next() {
                break;
            }
        }

        Ok::<String, Box<dyn std::error::Error + Send + Sync>>(content)
    })
    .await??;

    Ok(out)
}

pub async fn get_content_from_pdf(
    file_path: impl AsRef<Path>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let file_path = file_path.as_ref().to_path_buf();
    let bytes = tokio::fs::read(&file_path).await?;
    let out =
        tokio::task::spawn_blocking(move || pdf_extract::extract_text_from_mem(bytes.as_slice()))
            .await??;
    Ok(out)
}

pub fn extract_text_from_srt(
    srt_content: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut transcript = String::new();
    let lines: Vec<&str> = srt_content.lines().collect();

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();

        if line.is_empty() {
            i += 1;
            continue;
        }

        // Skip numeric counter lines
        if line.chars().all(|c| c.is_ascii_digit()) {
            i += 1;
            continue;
        }

        // Skip timing lines
        if line.contains("-->") {
            i += 1;
            continue;
        }

        // Subtitle text
        let cleaned_text = clean_subtitle_text(line);
        if !cleaned_text.is_empty() {
            transcript.push_str(&cleaned_text);
            transcript.push(' ');
        }

        i += 1;
    }

    if transcript.trim().is_empty() {
        Err("No text content found in the SRT file".into())
    } else {
        Ok(transcript.trim().to_string())
    }
}

pub fn extract_text_from_html(
    html_content: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let document = Html::parse_document(html_content);

    // Selectors for content-rich elements
    let selectors = [
        "article p",
        "article h1",
        "article h2",
        "article h3",
        "article h4",
        "article h5",
        "article h6",
        "article li",
        "main p",
        "main h1",
        "main h2",
        "main h3",
        "main h4",
        "main h5",
        "main h6",
        "main li",
        ".content p",
        ".post p",
        ".entry-content p",
        "body > p",
        "body > h1",
        "body > h2",
        "body > h3",
    ];

    let mut extracted_text = String::new();
    let mut seen_text = std::collections::HashSet::new();

    for selector_str in selectors {
        let selector = Selector::parse(selector_str).unwrap();
        for element in document.select(&selector) {
            let text = element
                .text()
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
                .to_string();
            if !text.is_empty() && !seen_text.contains(&text) {
                extracted_text.push_str(&text);
                extracted_text.push('\n');
                seen_text.insert(text);
            }
        }
    }

    // If still empty, fall back to all paragraphs in the body
    if extracted_text.is_empty() {
        let selector = Selector::parse("p").unwrap();
        for element in document.select(&selector) {
            let text = element
                .text()
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
                .to_string();
            if !text.is_empty() && !seen_text.contains(&text) {
                extracted_text.push_str(&text);
                extracted_text.push('\n');
                seen_text.insert(text);
            }
        }
    }

    if extracted_text.is_empty() {
        Err("No text content found in the HTML".into())
    } else {
        Ok(extracted_text)
    }
}

static SUBTITLE_PAIRS: &[(&str, &str)] = &[
    ("<c>", ""),
    ("</c>", ""),
    ("<i>", ""),
    ("</i>", ""),
    ("<b>", ""),
    ("</b>", ""),
    ("<u>", ""),
    ("</u>", ""),
    ("&amp;", "&"),
    ("&lt;", "<"),
    ("&gt;", ">"),
    ("&quot;", "\""),
    ("&#39;", "'"),
    ("<v ", ""),
    (">", " "),
];

static SUBTITLE_CLEANER: LazyLock<(AhoCorasick, Vec<&'static str>)> = LazyLock::new(|| {
    let patterns: Vec<&str> = SUBTITLE_PAIRS.iter().map(|(p, _)| *p).collect();
    let replacements: Vec<&str> = SUBTITLE_PAIRS.iter().map(|(_, r)| *r).collect();
    let ac = AhoCorasick::new(patterns).expect("Failed to build AhoCorasick automaton");
    (ac, replacements)
});

pub fn clean_subtitle_text(text: &str) -> String {
    let (ac, replacements) = &*SUBTITLE_CLEANER;
    ac.replace_all(text, replacements).trim().to_string()
}

use unicode_segmentation::UnicodeSegmentation;

pub fn get_expression_list_from_content(text: &str) -> HashMap<String, (usize, Option<String>)> {
    let mut expression_list: HashMap<String, (usize, Option<String>)> = HashMap::new();

    for sentence in text.unicode_sentences() {
        let cleaned_sentence = sentence.trim().replace('\n', " ");
        if cleaned_sentence.is_empty() {
            continue;
        }

        for word in cleaned_sentence.split_word_bounds() {
            if is_word(word) {
                let word_lower = word.to_lowercase();
                let entry = expression_list.entry(word_lower).or_insert((0, None));
                entry.0 += 1;
                // Store the first sentence we encounter as the context
                if entry.1.is_none() {
                    entry.1 = Some(cleaned_sentence.clone());
                }
            }
        }
    }
    expression_list
}

fn is_word(word: &str) -> bool {
    !word.is_empty() && word.chars().all(char::is_alphabetic)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_expression_list_from_content() {
        let text = "This is a sentence. This is another one!";
        let expression_list = get_expression_list_from_content(text);

        assert_eq!(expression_list.get("this").unwrap().0, 2);
        assert_eq!(expression_list.get("sentence").unwrap().0, 1);
        assert_eq!(
            expression_list.get("this").unwrap().1,
            Some("This is a sentence.".to_string())
        );
        assert_eq!(
            expression_list.get("another").unwrap().1,
            Some("This is another one!".to_string())
        );
    }

    #[test]
    fn test_is_word() {
        assert!(is_word("hello"));
        assert!(!is_word("hello123"));
        assert!(!is_word(""));
        assert!(!is_word("!"));
    }

    #[test]
    fn test_extract_text_from_srt() {
        let srt = "1\n00:00:20,000 --> 00:00:24,400\nHello <i>world</i>!\n\n2\n00:00:24,500 --> 00:00:28,000\nThis is a test.";
        let result = extract_text_from_srt(srt).unwrap();
        assert_eq!(result, "Hello world! This is a test.");
    }

    #[test]
    fn test_extract_text_from_html() {
        let html = "<html><body><article><h1>Title</h1><p>Paragraph 1</p></article></body></html>";
        let result = extract_text_from_html(html).unwrap();
        assert!(result.contains("Title"));
        assert!(result.contains("Paragraph 1"));
    }
}
