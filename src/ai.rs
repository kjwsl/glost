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
struct LemmaOutput {
    lemma: String,
}

pub async fn lemmatize_word(
    word: &str,
    context: &str,
    lang: Language,
    model: &str,
    url: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let client = reqwest::Client::new();
    
    let prompt = format!(
        "Return the dictionary form (lemma) of the {} word '{}' found in this sentence: '{}'. \
        Respond with only a JSON object containing the field 'lemma'. \
        Example: {{ \"lemma\": \"talo\" }}",
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
    let lemma_output: LemmaOutput = serde_json::from_str(&ollama_res.response)?;

    Ok(lemma_output.lemma.to_lowercase())
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
