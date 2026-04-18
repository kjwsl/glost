use crate::Language;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Serialize, Deserialize, Debug)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    format: String,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

#[derive(Deserialize)]
pub struct ExpressionAnalysis {
    pub lemma: String,
    pub meaning: String,
    pub cefr: Option<String>,
    pub grammar: Option<String>,
}

pub async fn lemmatize_sentence(
    sentence: &str,
    lang: Language,
    model: &str,
    url: &str,
) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
    let client = reqwest::Client::new();

    let prompt = format!(
        "Act as an expert {lang} linguist. Break down this sentence into individual words/phrases (lemmas): '{sentence}'.

        Rules:
        1. Return ONLY the base/dictionary forms
        2. For multi-word idioms (e.g., 'by the way', 'take into account'), keep as one unit
        3. Ignore common function words like articles and prepositions unless they change meaning
        4. Use null for words you cannot determine

        Respond ONLY with a JSON array of strings: [\"word1\", \"word2\", \"...\"]",
        lang = lang, sentence = sentence
    );

    let request = OllamaRequest {
        model: model.to_string(),
        prompt,
        stream: false,
        format: "json".to_string(),
    };

    let res = client
        .post(format!("{}/api/generate", url))
        .json(&request)
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(format!("Ollama API error: {}", res.status()).into());
    }

    let ollama_res: OllamaResponse = res.json().await?;
    let lemmas: Vec<String> = serde_json::from_str(&ollama_res.response)?;

    Ok(lemmas)
}

pub async fn analyze_expression(
    expression: &str,
    context: &str,
    lang: Language,
    model: &str,
    url: &str,
) -> Result<ExpressionAnalysis, Box<dyn Error + Send + Sync>> {
    let client = reqwest::Client::new();

    let prompt = format!(
        "Act as an expert linguist specializing in {lang} and English. \
        Analyze the word '{expression}' from this context: '{context}'.

        Your task:
        1. Identify the dictionary form (lemma)
        2. Provide a concise English meaning for THIS context
        3. Estimate CEFR level (A1-C2)
        4. Brief grammar note (case, tense, mood, or 'idiom')

        Respond ONLY with JSON:
        {{\"lemma\":\"...\",\"meaning\":\"...\",\"cefr\":\"...\",\"grammar\":\"...\"}}",
        lang = lang, expression = expression, context = context
    );

    let request = OllamaRequest {
        model: model.to_string(),
        prompt,
        stream: false,
        format: "json".to_string(),
    };

    let res = client
        .post(format!("{}/api/generate", url))
        .json(&request)
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(format!("Ollama API error: {}", res.status()).into());
    }

    let ollama_res: OllamaResponse = res.json().await?;
    let analysis: ExpressionAnalysis = serde_json::from_str(&ollama_res.response)?;

    Ok(analysis)
}

pub async fn lemmatize_expression(
    expression: &str,
    context: &str,
    lang: Language,
    model: &str,
    url: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let analysis = analyze_expression(expression, context, lang, model, url).await?;
    Ok(analysis.lemma.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_request_serialization() {
        let req = OllamaRequest {
            model: "llama3".to_string(),
            prompt: "test".to_string(),
            stream: false,
            format: "json".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"model\":\"llama3\""));
        assert!(json.contains("\"stream\":false"));
    }
}