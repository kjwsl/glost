use crate::Language;
use yt_dlp::Downloader;
use yt_dlp::client::deps::Libraries;
use std::path::PathBuf;
use tempfile::tempdir;

pub async fn get_youtube_transcript(
    video_url: &str,
    lang: Language,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let lang_code = lang.to_lang_code();
    
    // We assume yt-dlp and ffmpeg are in the PATH or we use a temporary location
    // For simplicity, we'll try to use the ones from the system PATH first.
    // If not found, the user should install them.
    // The crate can download them, but let's try to be lightweight.
    
    let dir = tempdir()?;
    let lib_path = dir.path().join("libs");
    std::fs::create_dir_all(&lib_path)?;
    
    // Libraries::new requires paths to the binaries.
    // Let's try to find them in the system.
    let yt_dlp_path = which::which("yt-dlp").unwrap_or_else(|_| PathBuf::from("yt-dlp"));
    let ffmpeg_path = which::which("ffmpeg").unwrap_or_else(|_| PathBuf::from("ffmpeg"));
    
    let libraries = Libraries::new(yt_dlp_path, ffmpeg_path);
    let downloader = Downloader::builder(libraries, dir.path().to_str().unwrap()).build().await?;
    
    let video = downloader.fetch_video_infos(video_url).await?;
    
    let sub_filename = format!("sub_{}.srt", lang_code);
    let sub_path = downloader
        .download_subtitle(&video, &lang_code, &sub_filename, true)
        .await?;

    let content = tokio::fs::read_to_string(&sub_path).await?;
    
    // The downloader usually saves as .srt or .vtt depending on what's available
    let ext = sub_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext {
        "vtt" => extract_text_from_vtt(&content),
        "srt" => crate::content::extract_text_from_srt(&content),
        _ => {
            // If it's something else, try to parse it as SRT as a fallback
            crate::content::extract_text_from_srt(&content)
        }
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

pub fn extract_text_from_vtt(vtt_content: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut transcript = String::new();
    let lines = vtt_content.lines();
    let mut is_content = false;

    for line in lines {
        let line = line.trim();
        if line.is_empty() || line == "WEBVTT" || line.starts_with("Kind:") || line.starts_with("Language:") {
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
        assert_eq!(extract_video_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ").unwrap(), "dQw4w9WgXcQ");
        assert_eq!(extract_video_id("https://youtu.be/dQw4w9WgXcQ").unwrap(), "dQw4w9WgXcQ");
    }
}
