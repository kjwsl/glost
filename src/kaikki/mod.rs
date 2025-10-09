pub mod types;

pub use types::*;

pub async fn get_from_kaikki(
    word: &str,
) -> Result<Vec<Entry>, Box<dyn std::error::Error + Send + Sync>> {
    if word.is_empty() {
        return Err("Word is empty".into());
    }
    
    // Try the original word first
    match try_get_from_kaikki_with_word(word).await {
        Ok(entries) if !entries.is_empty() => return Ok(entries),
        _ => {}
    }
    
    // If original word fails or returns no entries, try lowercase
    let lower_word = word.to_lowercase();
    if lower_word != word {
        try_get_from_kaikki_with_word(&lower_word).await
    } else {
        Ok(vec![])
    }
}

async fn try_get_from_kaikki_with_word(
    word: &str,
) -> Result<Vec<Entry>, Box<dyn std::error::Error + Send + Sync>> {
    let ch1 = word.chars().next().unwrap();
    let ch2_opt = word.chars().nth(1);

    let part2 = match ch2_opt {
        Some(ch2) => format!("{ch1}{ch2}"),
        None => format!("{ch1}"),
    };

    let url = format!(
        "https://kaikki.org/dictionary/All%20languages%20combined/meaning/{ch1}/{part2}/{word}.jsonl"
    );

    let mut retries = 3;
    let mut delay = std::time::Duration::from_millis(100);

    let resp = loop {
        match reqwest::get(&url).await {
            Ok(resp) => break resp,
            Err(e) => {
                if retries == 0 {
                    return Err(e.into());
                }
                retries -= 1;
                tokio::time::sleep(delay).await;
                delay *= 2;
            }
        }
    };

    if !resp.status().is_success() {
        return Err(format!("Failed to get entry from kaikki.org: {}", resp.status()).into());
    }

    let text = resp.text().await?;
    let mut entries = vec![];

    for line in text.lines() {
        if line.is_empty() {
            continue;
        }
        let entry = match serde_json::from_str::<Entry>(line) {
            Ok(e) => e,
            Err(e) => return Err(format!("Failed to parse entry: {e}").into()),
        };
        entries.push(entry);
    }

    Ok(entries)
}