use crate::Language;
use std::{
    collections::{HashMap, HashSet},
    fs,
    str::FromStr,
};

#[derive(Debug, Clone)]
pub struct FilterList {
    words_by_language: HashMap<Language, HashSet<String>>,
}

impl Default for FilterList {
    fn default() -> Self {
        Self::new()
    }
}

impl FilterList {
    pub fn new() -> Self {
        Self {
            words_by_language: HashMap::new(),
        }
    }

    pub fn load(file_path: &str) -> Result<Self, std::io::Error> {
        match fs::read_to_string(file_path) {
            Ok(content) => {
                let mut words_by_language = HashMap::new();

                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }

                    // Format: "language:word" or just "word" (defaults to English)
                    if let Some((lang_str, word)) = line.split_once(':') {
                        if let Ok(lang) = Language::from_str(lang_str.trim()) {
                            words_by_language
                                .entry(lang)
                                .or_insert_with(HashSet::new)
                                .insert(word.trim().to_lowercase());
                        }
                    } else {
                        // Default to English for backward compatibility
                        words_by_language
                            .entry(Language::English)
                            .or_insert_with(HashSet::new)
                            .insert(line.to_lowercase());
                    }
                }

                Ok(Self { words_by_language })
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::new()),
            Err(e) => Err(e),
        }
    }

    pub fn save(&self, file_path: &str) -> Result<(), std::io::Error> {
        let mut lines = Vec::new();

        // Add header comment
        lines.push(
            "# Filter list - Format: language:word or just word (defaults to English)".to_string(),
        );
        lines.push("".to_string());

        // Sort languages for consistent output
        let mut languages: Vec<_> = self.words_by_language.keys().collect();
        languages.sort_by_key(|a| a.to_string());

        for &language in languages {
            if let Some(words) = self.words_by_language.get(&language) {
                let mut word_list: Vec<_> = words.iter().collect();
                word_list.sort();

                for word in word_list {
                    if language == Language::English {
                        // For backward compatibility, don't prefix English words
                        lines.push(word.clone());
                    } else {
                        lines.push(format!("{}:{}", language, word));
                    }
                }
            }
        }

        fs::write(file_path, lines.join("\n"))
    }

    pub fn add(&mut self, word: String, language: Language) {
        self.words_by_language
            .entry(language)
            .or_default()
            .insert(word.to_lowercase());
    }

    pub fn remove(&mut self, word: &str, language: Language) -> bool {
        if let Some(words) = self.words_by_language.get_mut(&language) {
            words.remove(&word.to_lowercase())
        } else {
            false
        }
    }

    pub fn contains(&self, word: &str, language: Language) -> bool {
        self.words_by_language
            .get(&language)
            .map(|words| words.contains(&word.to_lowercase()))
            .unwrap_or(false)
    }

    pub fn clear_language(&mut self, language: Language) {
        self.words_by_language.remove(&language);
    }

    pub fn list(&self, language: Option<Language>) -> Vec<(Language, String)> {
        let mut result = Vec::new();

        match language {
            Some(lang) => {
                if let Some(words) = self.words_by_language.get(&lang) {
                    let mut word_list: Vec<_> = words.iter().cloned().collect();
                    word_list.sort();
                    for word in word_list {
                        result.push((lang, word));
                    }
                }
            }
            None => {
                let mut languages: Vec<_> = self.words_by_language.keys().collect();
                languages.sort_by_key(|a| a.to_string());

                for &lang in languages {
                    if let Some(words) = self.words_by_language.get(&lang) {
                        let mut word_list: Vec<_> = words.iter().cloned().collect();
                        word_list.sort();
                        for word in word_list {
                            result.push((lang, word));
                        }
                    }
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_filter_list_basic() {
        let mut filter = FilterList::new();
        filter.add("talo".to_string(), Language::Finnish);
        
        assert!(filter.contains("talo", Language::Finnish));
        assert!(filter.contains("TALO", Language::Finnish)); // Case insensitive
        assert!(!filter.contains("talo", Language::English)); // Language specific
    }

    #[test]
    fn test_filter_list_load_save() -> Result<(), std::io::Error> {
        let mut tmp = NamedTempFile::new()?;
        writeln!(tmp, "finnish:kissa")?;
        writeln!(tmp, "koira")?; // Defaults to English
        
        let mut filter = FilterList::load(tmp.path().to_str().unwrap())?;
        assert!(filter.contains("kissa", Language::Finnish));
        assert!(filter.contains("koira", Language::English));
        
        filter.add("omena".to_string(), Language::Finnish);
        filter.save(tmp.path().to_str().unwrap())?;
        
        let filter2 = FilterList::load(tmp.path().to_str().unwrap())?;
        assert!(filter2.contains("omena", Language::Finnish));
        
        Ok(())
    }
}
