use crate::Language;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    format: Option<String>,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

#[derive(Deserialize)]
pub struct WordAnalysis {
    pub lemma: String,
    pub cefr: Option<String>,
    pub grammar: Option<String>,
}

pub async fn analyze_word(
    word: &str,
    context: &str,
    lang: Language,
    model: &str,
    url: &str,
) -> Result<WordAnalysis, Box<dyn Error + Send + Sync>> {
    let client = reqwest::Client::new();
    
    let prompt = format!(
        "Analyze the {} word '{}' found in this sentence: '{}'. \
        Return its dictionary form (lemma), its estimated CEFR difficulty level (A1-C2), \
        and a brief grammar explanation (especially the grammatical case or tense if applicable in the context). \
        Respond with only a JSON object containing 'lemma', 'cefr', and 'grammar' fields. \
        Example: {{ \"lemma\": \"talo\", \"cefr\": \"A1\", \"grammar\": \"inessive singular ('in the house')\" }}",
        lang, word, context
    );

    let request = OllamaRequest {
        model: model.to_string(),
        prompt,
        stream: false,
        format: Some("json".to_string()),
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
    let analysis: WordAnalysis = serde_json::from_str(&ollama_res.response)?;

    Ok(analysis)
}

pub async fn lemmatize_word(
    word: &str,
    context: &str,
    lang: Language,
    model: &str,
    url: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let analysis = analyze_word(word, context, lang, model, url).await?;
    Ok(analysis.lemma.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_request_serialization() {
        let request = OllamaRequest {
            model: "llama3".to_string(),
            prompt: "test".to_string(),
            stream: false,
            format: Some("json".to_string()),
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"model\":\"llama3\""));
        assert!(json.contains("\"format\":\"json\""));
    }
}
