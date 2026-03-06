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

pub fn get_word_list_from_content(text: &str) -> HashMap<String, usize> {
    let mut word_list = HashMap::new();
    for word in text.split_whitespace() {
        if is_word(word) {
            *word_list.entry(word.to_string()).or_insert(0) += 1;
        }
    }
    word_list
}

fn is_word(word: &str) -> bool {
    word.chars().all(char::is_alphabetic)
}
