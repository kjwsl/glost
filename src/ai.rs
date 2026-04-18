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
        lang: Language,
    ) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let prompt = format!(
            "Act as an expert {lang} linguist. Break down this sentence into individual words/phrases (lemmas): '{sentence}'.

            Rules:
            1. Return ONLY the base/dictionary forms
            2. For multi-word idioms (e.g., 'by the way', 'take into account'), keep as one unit
            3. Ignore common function words like articles and prepositions unless they change meaning
            4. Use null for words you cannot determine

            Respond ONLY with a JSON array of strings: [\"word1\", \"word2\", \"...\"]"
        );

        self.execute_prompt(prompt).await
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
