use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub static YOUTUBE_API_KEY: Lazy<String> = Lazy::new(|| {
    std::env::var("YOUTUBE_API_KEY").expect("YOUTUBE_API_KEY must be set")
});

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CaptionListResponse {
    items: Vec<CaptionItem>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CaptionItem {
    id: String,
    snippet: CaptionSnippet,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CaptionSnippet {
    language: String,
    name: String,
    #[serde(rename = "trackKind")]
    track_kind: String,
}

pub async fn fetch_transcript(
    video_id: &str,
    lang: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    println!("Fetching transcript for video ID: {}, language: {}", video_id, lang);
    let client = Client::new();
    let captions_url = format!(
        "https://www.googleapis.com/youtube/v3/captions?part=snippet&videoId={}&key={}",
        video_id, YOUTUBE_API_KEY.clone()
    );

    let response = client
        .get(&captions_url)
        .send()
        .await?
        .json::<CaptionListResponse>()
        .await?;

    let caption = response
        .items
        .into_iter()
        .find(|item| item.snippet.language == lang && item.snippet.track_kind != "forced")
        .ok_or("No suitable captions found")?;

    let caption_url = format!(
        "https://www.googleapis.com/youtube/v3/captions/{}?tfmt=srt&key={}",
        caption.id, YOUTUBE_API_KEY.clone()
    );

    let caption_text = client
        .get(&caption_url)
        .header("Accept", "text/plain")
        .send()
        .await?
        .text()
        .await?;

    let transcript = parse_srt(&caption_text)?;

    Ok(transcript)
}

fn parse_srt(srt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut transcript = String::new();
    for block in srt.split("\n\n") {
        let lines: Vec<&str> = block.lines().collect();
        if lines.len() >= 3 {
            let text = lines[2..].join(" ");
            transcript.push_str(&text);
            transcript.push(' ');
        }
    }
    Ok(transcript.trim().to_string())
}
