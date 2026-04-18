use crate::Language;
use std::path::PathBuf;
use std::process::Command as StdCommand;
use tokio::fs;
use tokio::process::Command;

pub async fn get_youtube_transcript(
    video_url: &str,
    lang: Language,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    check_yt_dlp_installed()?;

    let lang_code = lang.to_lang_code();
    let video_id = extract_video_id(video_url).ok_or("Could not extract video ID")?;

    let output = Command::new("yt-dlp")
        .args([
            "--write-subs",
            "--write-auto-subs",
            "--sub-lang",
            &lang_code,
            "--skip-download",
            "--output",
            &format!("{}.%(ext)s", video_id),
            video_url,
        ])
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("Video unavailable") {
            return Err("Video unavailable".into());
        }
        return Err(format!("yt-dlp failed: {}", stderr).into());
    }

    let vtt_path = PathBuf::from(format!("{}.{}.vtt", video_id, lang_code));
    let srt_path = PathBuf::from(format!("{}.{}.srt", video_id, lang_code));
    let langcode_vtt_path = PathBuf::from(format!("{}.vtt", video_id));
    let langcode_srt_path = PathBuf::from(format!("{}.srt", video_id));

    let actual_path = if vtt_path.exists() {
        vtt_path
    } else if srt_path.exists() {
        srt_path
    } else if langcode_vtt_path.exists() {
        langcode_vtt_path
    } else if langcode_srt_path.exists() {
        langcode_srt_path
    } else {
        return Err(format!("No subtitles found for video {} in {}. Try a different language or check if video has captions.", video_id, lang_code).into());
    };

    let content = fs::read_to_string(&actual_path).await?;

    let _ = fs::remove_file(&actual_path).await;

    let ext = actual_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("srt");
    match ext {
        "vtt" => extract_text_from_vtt(&content),
        _ => crate::content::extract_text_from_srt(&content),
    }
}

fn check_yt_dlp_installed() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let result = StdCommand::new("yt-dlp").arg("--version").output();

    match result {
        Ok(output) if output.status.success() => Ok(()),
        Ok(_) => Err("yt-dlp found but --version failed".into()),
        Err(_) => Err(r#"yt-dlp not found. Please install it:
  pip install yt-dlp
  # or
  brew install yt-dlp
  # or
  pipx install yt-dlp"#
            .into()),
    }
}

pub fn extract_video_id(url: &str) -> Option<String> {
    if url.len() == 11 && !url.contains('/') {
        return Some(url.to_string());
    }
    let patterns = ["v=", "be/", "embed/", "shorts/"];
    for pattern in patterns {
        if let Some(idx) = url.find(pattern) {
            let start = idx + pattern.len();
            let end = url[start..].find('&').unwrap_or(url[start..].len());
            return Some(url[start..start + end].to_string());
        }
    }
    None
}

pub fn extract_text_from_vtt(
    vtt_content: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut transcript = String::new();
    let lines = vtt_content.lines();
    let mut is_content = false;

    for line in lines {
        let line = line.trim();
        if line.is_empty()
            || line == "WEBVTT"
            || line.starts_with("Kind:")
            || line.starts_with("Language:")
        {
            continue;
        }

        if line.contains("-->") {
            is_content = true;
            continue;
        }

        if is_content {
            let cleaned = crate::content::clean_subtitle_text(line);
            if !cleaned.is_empty() {
                if !transcript.is_empty() && !transcript.ends_with(' ') {
                    transcript.push(' ');
                }
                transcript.push_str(&cleaned);
            }
        }
    }

    if transcript.trim().is_empty() {
        Err("No text content found in the VTT subtitles".into())
    } else {
        Ok(transcript.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_video_id() {
        assert_eq!(
            extract_video_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ").unwrap(),
            "dQw4w9WgXcQ"
        );
        assert_eq!(
            extract_video_id("https://youtu.be/dQw4w9WgXcQ").unwrap(),
            "dQw4w9WgXcQ"
        );
    }
}
