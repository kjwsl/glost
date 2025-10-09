use std::{collections::HashMap, fs, path::Path};
use epub::doc::EpubDoc;

pub fn get_content_from_file(
    file_path: impl AsRef<Path>,
) -> Result<String, Box<dyn std::error::Error>> {
    let file_path = file_path.as_ref();
    let ext = file_path
        .extension()
        .ok_or("File has no extension")?
        .to_str()
        .unwrap();
        
    match ext {
        "epub" => get_content_from_epub(file_path),
        "pdf" => get_content_from_pdf(file_path),
        "txt" => Ok(fs::read_to_string(file_path)?),
        _ => Err("Unsupported file extension".into()),
    }
}

pub fn get_content_from_epub(
    file_path: impl AsRef<Path>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut doc = EpubDoc::new(file_path)?;
    let mut content = String::new();

    while let Some((chapter_content, _mime_type)) = doc.get_current_str() {
        content.push_str(&chapter_content);
        if !doc.go_next() {
            break;
        }
    }

    Ok(content)
}

pub fn get_content_from_pdf(
    file_path: impl AsRef<Path>,
) -> Result<String, Box<dyn std::error::Error>> {
    let bytes = fs::read(file_path)?;
    let out = pdf_extract::extract_text_from_mem(bytes.as_slice())?;
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