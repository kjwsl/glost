use epub::doc::EpubDoc;
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

use unicode_segmentation::UnicodeSegmentation;

pub fn get_word_list_from_content(text: &str) -> HashMap<String, (usize, Option<String>)> {
    let mut word_list: HashMap<String, (usize, Option<String>)> = HashMap::new();

    for sentence in text.unicode_sentences() {
        let cleaned_sentence = sentence.trim().replace('\n', " ");
        if cleaned_sentence.is_empty() {
            continue;
        }

        for word in cleaned_sentence.split_word_bounds() {
            if is_word(word) {
                let word_lower = word.to_lowercase();
                let entry = word_list.entry(word_lower).or_insert((0, None));
                entry.0 += 1;
                // Store the first sentence we encounter as the context
                if entry.1.is_none() {
                    entry.1 = Some(cleaned_sentence.clone());
                }
            }
        }
    }
    word_list
}

fn is_word(word: &str) -> bool {
    !word.is_empty() && word.chars().all(char::is_alphabetic)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_word_list_from_content() {
        let text = "This is a sentence. This is another one!";
        let word_list = get_word_list_from_content(text);

        assert_eq!(word_list.get("this").unwrap().0, 2);
        assert_eq!(word_list.get("sentence").unwrap().0, 1);
        assert_eq!(
            word_list.get("this").unwrap().1,
            Some("This is a sentence.".to_string())
        );
        assert_eq!(
            word_list.get("another").unwrap().1,
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
}
