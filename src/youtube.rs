use std::error::Error;
use std::sync::LazyLock;

use aho_corasick::AhoCorasick;
use url::Url;
use yt_transcript_fetcher::fetch_transcript;

use crate::Language;

pub async fn get_youtube_transcript(
    video_url: &str,
    lang: Language,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let video_id = extract_video_id(video_url)?;

    fetch_transcript(&video_id, lang.to_lang_code()).await
}

fn extract_video_id(url: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    let parsed_url = Url::parse(url)?;

    match parsed_url.host_str() {
        Some("www.youtube.com") | Some("youtube.com") | Some("m.youtube.com") => {
            if let Some(query) = parsed_url.query() {
                for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
                    if key == "v" {
                        return Ok(value.to_string());
                    }
                }
            }
            Err("Could not find video ID in YouTube URL".into())
        }
        Some("youtu.be") => {
            let path = parsed_url.path();
            if path.starts_with('/') && path.len() > 1 {
                // Handle query parameters in youtu.be URLs
                let video_id = &path[1..];
                if let Some(question_mark) = video_id.find('?') {
                    Ok(video_id[..question_mark].to_string())
                } else {
                    Ok(video_id.to_string())
                }
            } else {
                Err("Invalid youtu.be URL format".into())
            }
        }
        _ => Err("Not a valid YouTube URL".into()),
    }
}

static SUBTITLE_PAIRS: &[(&str, &str)] = &[
    ("<c>", ""),
    ("</c>", ""),
    ("<i>", ""),
    ("</i>", ""),
    ("<b>", ""),
    ("</b>", ""),
    ("<u>", ""),
    ("</u>", ""),
    ("&amp;", "&"),
    ("&lt;", "<"),
    ("&gt;", ">"),
    ("&quot;", "\""),
    ("&#39;", "'"),
    ("<v ", ""),
    (">", " "),
];

static SUBTITLE_CLEANER: LazyLock<(AhoCorasick, Vec<&'static str>)> = LazyLock::new(|| {
    let patterns: Vec<&str> = SUBTITLE_PAIRS.iter().map(|(p, _)| *p).collect();
    let replacements: Vec<&str> = SUBTITLE_PAIRS.iter().map(|(_, r)| *r).collect();
    let ac = AhoCorasick::new(patterns).expect("Failed to build AhoCorasick automaton");
    (ac, replacements)
});

pub fn extract_text_from_vtt(vtt_content: &str) -> Result<String, Box<dyn Error>> {
    let mut transcript = String::new();
    let lines: Vec<&str> = vtt_content.lines().collect();

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();

        // Skip VTT headers and timing lines
        if line.starts_with("WEBVTT")
            || line.starts_with("NOTE")
            || line.contains("-->")
            || line.is_empty()
        {
            i += 1;
            continue;
        }

        // Skip lines that look like timestamps
        if line.chars().next().is_some_and(|c| c.is_ascii_digit())
            && (line.contains(':') || line.contains('.'))
        {
            i += 1;
            continue;
        }

        // This should be subtitle text
        if !line.is_empty() {
            let cleaned_text = clean_subtitle_text(line);
            if !cleaned_text.is_empty() {
                transcript.push_str(&cleaned_text);
                transcript.push(' ');
            }
        }

        i += 1;
    }

    if transcript.trim().is_empty() {
        Err("No text content found in the subtitle file".into())
    } else {
        Ok(transcript.trim().to_string())
    }
}

fn clean_subtitle_text(text: &str) -> String {
    // Remove HTML tags, clean up subtitle text, and remove VTT formatting
    let (ac, replacements) = &*SUBTITLE_CLEANER;
    ac.replace_all(text, replacements).trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_video_id() {
        let test_cases = vec![
            ("https://www.youtube.com/watch?v=dQw4w9WgXcQ", "dQw4w9WgXcQ"),
            ("https://youtu.be/dQw4w9WgXcQ", "dQw4w9WgXcQ"),
            ("https://youtu.be/dQw4w9WgXcQ?si=abc123", "dQw4w9WgXcQ"),
            (
                "https://youtube.com/watch?v=dQw4w9WgXcQ&t=60s",
                "dQw4w9WgXcQ",
            ),
        ];

        for (url, expected_id) in test_cases {
            assert_eq!(extract_video_id(url).unwrap(), expected_id);
        }
    }

    #[test]
    fn test_clean_subtitle_text() {
        let input = "<c>Hello &amp; welcome to <i>YouTube</i> &#39;transcripts&#39;</c>";
        let expected = "Hello & welcome to YouTube 'transcripts'";
        assert_eq!(clean_subtitle_text(input), expected);
    }
}
