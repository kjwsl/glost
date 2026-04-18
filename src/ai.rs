use crate::Language;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Serialize, Debug)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: String,
    stream: bool,
    format: &'a str,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

#[derive(Deserialize, Debug)]
pub struct ExpressionAnalysis {
    pub lemma: String,
    pub meaning: String,
    pub cefr: Option<String>,
    pub grammar: Option<String>,
}

/// A structured client for interacting with the Ollama linguistics API.
pub struct OllamaLinguist {
    client: reqwest::Client,
    base_url: String,
    model: String,
}

impl OllamaLinguist {
    /// Creates a new OllamaLinguist, initializing the connection pool once.
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.into(),
            model: model.into(),
        }
    }

    /// Centralized helper method to handle the repetitive HTTP and parsing logic.
    async fn execute_prompt<T: for<'de> Deserialize<'de>>(
        &self,
        prompt: String,
    ) -> Result<T, Box<dyn Error + Send + Sync>> {
        let request = OllamaRequest {
            model: &self.model,
            prompt,
            stream: false,
            format: "json",
        };

        let res = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .json(&request)
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(format!("Ollama API error: {}", res.status()).into());
        }

        let ollama_res: OllamaResponse = res.json().await?;
        let parsed_data: T = serde_json::from_str(&ollama_res.response)?;

        Ok(parsed_data)
    }

    pub async fn lemmatize_sentence(
        &self,
        sentence: &str,
        _lang: Language,
    ) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let prompt = format!(
            "Extract ALL words from this sentence as a flat list: \"{}\"

Rules:
- Return EVERY content word, one by one in order
- Keep multi-word phrases together (e.g., \"by the way\" stays as 3 words)
- Include everything, do NOT filter articles/prepositions

Format: ONLY a JSON array with double quotes around each word like [\"word1\", \"word2\", \"word3\"]",
            sentence
        );

        let request = OllamaRequest {
            model: &self.model,
            prompt,
            stream: false,
            format: "json",
        };

        let res = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .json(&request)
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(format!("Ollama API error: {}", res.status()).into());
        }

        let ollama_res: OllamaResponse = res.json().await?;

        // Try to parse as JSON - handle various formats
        let parsed: serde_json::Value =
            serde_json::from_str(&ollama_res.response).or_else(|_| {
                if let Some(start) = ollama_res.response.find('[') {
                    if let Some(end) = ollama_res.response.rfind(']') {
                        let json_part = &ollama_res.response[start..=end];
                        serde_json::from_str(json_part)
                    } else {
                        Err(serde_json::Error::io(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "no closing bracket",
                        )))
                    }
                } else {
                    Err(serde_json::Error::io(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "no array found",
                    )))
                }
            })?;

        // Handle the response format - could be array, object with array values, or string
        let strings = match parsed {
            serde_json::Value::Array(arr) => arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
            serde_json::Value::Object(obj) => {
                // Check if this is a map of name->value pairs where value might be a JSON array string
                let mut result = Vec::new();
                for (_, v) in obj {
                    match v {
                        serde_json::Value::Array(arr) => {
                            for item in arr {
                                if let Some(s) = item.as_str() {
                                    result.push(s.to_string());
                                }
                            }
                        }
                        serde_json::Value::String(s) => {
                            // Try to parse as JSON array if it looks like one
                            if s.starts_with('[') {
                                if let Ok(parsed_inner) = serde_json::from_str::<Vec<String>>(&s) {
                                    result.extend(parsed_inner);
                                } else {
                                    result.push(s.clone());
                                }
                            } else {
                                result.push(s.clone());
                            }
                        }
                        _ => {}
                    }
                }
                result
            }
            serde_json::Value::String(s) => Self::extract_words_from_text(&s),
            _ => {
                return Err(format!("Unexpected JSON type: {:?}", parsed).into());
            }
        };

        Ok(strings)
    }

    fn extract_words_from_text(text: &str) -> Vec<String> {
        let mut words = Vec::new();

        // Try to extract from [word] format
        let bracket_pattern = regex::Regex::new(r"\[([^\]]+)\]").unwrap();
        for cap in bracket_pattern.captures_iter(text) {
            if let Some(word) = cap.get(1) {
                let w = word.as_str().to_string();
                if !w.chars().all(|c| c == '♪' || c == ' ') {
                    words.push(w);
                }
            }
        }

        // Try to handle Python-style list like ['word1', 'word2']
        if words.is_empty() && text.starts_with('[') {
            let inner = text.trim_start_matches('[').trim_end_matches(']');
            // Split by comma and extract quoted strings
            let item_pattern = regex::Regex::new(r"'([^']*)'").unwrap();
            for cap in item_pattern.captures_iter(inner) {
                if let Some(word) = cap.get(1) {
                    let w = word.as_str().trim().to_string();
                    if !w.is_empty() {
                        words.push(w);
                    }
                }
            }
        }

        // If still no words, just split by whitespace
        if words.is_empty() {
            words = text
                .split_whitespace()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty() && s.chars().any(|c| c.is_alphabetic()))
                .collect();
        }

        words
    }

    pub async fn analyze_expression(
        &self,
        expression: &str,
        context: &str,
        lang: Language,
    ) -> Result<ExpressionAnalysis, Box<dyn Error + Send + Sync>> {
        let prompt = format!(
            "Act as an expert {lang} linguist. Analyze the word '{expression}' used in this context: '{context}'.

            Your task:
            1. Identify the dictionary form (lemma) - the base word this inflected form comes from
            2. CRITICAL: Look at the context and pick the ONE meaning that fits best. DO NOT guess - use the context to disambiguate.
            3. If the context is ambiguous, use the most common meaning
            4. CEFR level only if you can determine it reliably, otherwise omit
            5. Brief grammar note (case, tense, mood, or 'idiom') if relevant

            The meaning field should be 1-3 words max, directly describing what the word means IN THIS SPECIFIC CONTEXT.

            Respond ONLY with JSON (no extra text):
            {{\"lemma\":\"...\",\"meaning\":\"...\",\"cefr\":\"...\",\"grammar\":\"...\"}}"
        );

        self.execute_prompt(prompt).await
    }

    pub async fn lemmatize_expression(
        &self,
        expression: &str,
        context: &str,
        lang: Language,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let analysis = self.analyze_expression(expression, context, lang).await?;
        Ok(analysis.lemma.to_lowercase())
    }

    pub fn model_name(&self) -> &str {
        &self.model
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_request_serialization() {
        let req = OllamaRequest {
            model: "llama3",
            prompt: "test".to_string(),
            stream: false,
            format: "json",
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"model\":\"llama3\""));
        assert!(json.contains("\"stream\":false"));
    }
}
