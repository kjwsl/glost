use crate::Language;
use std::error::Error;
use std::path::Path;
use std::process::Command;

pub fn generate_audio_file(
    word: &str,
    lang: Language,
    output_dir: &Path,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let filename = format!("{}_{}.aiff", word, lang.to_lang_code());
    let output_path = output_dir.join(&filename);

    // On macOS, we can use the 'say' command which is very reliable for Finnish
    // if the voice is installed.
    #[cfg(target_os = "macos")]
    {
        let voice = match lang {
            Language::Finnish => "Satu", // Default Finnish voice on macOS
            Language::Swedish => "Oskar",
            _ => "",
        };

        let mut cmd = Command::new("say");
        if !voice.is_empty() {
            cmd.arg("-v").arg(voice);
        }
        cmd.arg(word).arg("-o").arg(&output_path);

        let status = cmd.status()?;
        if !status.success() {
            return Err("Failed to execute 'say' command".into());
        }
    }

    // On Linux, we'd use espeak-ng or similar
    #[cfg(target_os = "linux")]
    {
        let lang_code = match lang {
            Language::Finnish => "fi",
            Language::Swedish => "sv",
            _ => lang.to_lang_code(),
        };

        let status = Command::new("espeak-ng")
            .arg("-v")
            .arg(lang_code)
            .arg("-w")
            .arg(&output_path)
            .arg(word)
            .status()?;

        if !status.success() {
            return Err("Failed to execute 'espeak-ng' command".into());
        }
    }

    Ok(filename)
}
