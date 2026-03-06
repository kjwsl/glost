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
    let mut transcript = String::with_capacity(srt.len());
    for block in srt.split("\n\n") {
        let mut lines = block.lines().skip(2);
        if let Some(first_line) = lines.next() {
            transcript.push_str(first_line);
            for line in lines {
                transcript.push(' ');
                transcript.push_str(line);
            }
            transcript.push(' ');
        }
    }
    Ok(transcript.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn bench_parse_srt() {
        let mut srt = String::new();
        for i in 1..1000 {
            srt.push_str(&format!("{}\n00:00:0{:03},000 --> 00:00:0{:03},500\nLine 1 for block {}\nLine 2 for block {}\n\n", i, i % 1000, i % 1000, i, i));
        }

        let start = Instant::now();
        for _ in 0..1000 {
            let _ = parse_srt(&srt).unwrap();
        }
        let duration = start.elapsed();
        println!("Time taken for 1000 iterations: {:?}", duration);
    }

    #[test]
    fn test_parse_srt_correctness() {
        let srt = "1\n00:00:00,000 --> 00:00:01,000\nHello world\n\n2\n00:00:01,000 --> 00:00:02,000\nThis is a test\nwith two lines";
        let expected = "Hello world This is a test with two lines";
        assert_eq!(parse_srt(srt).unwrap(), expected);
    }
}
