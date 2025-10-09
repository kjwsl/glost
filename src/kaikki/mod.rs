pub mod types;

pub use types::*;

pub async fn get_from_kaikki(
    word: &str,
) -> Result<Vec<Entry>, Box<dyn std::error::Error + Send + Sync>> {
    if word.is_empty() {
        return Err("Word is empty".into());
    }
    let lower_word = word.to_lowercase();
    let ch1 = lower_word.chars().next().unwrap();
    let ch2_opt = lower_word.chars().nth(1);

    let part2 = match ch2_opt {
        Some(ch2) => format!("{ch1}{ch2}"),
        None => format!("{ch1}"),
    };

    let url = format!(
        "https://kaikki.org/dictionary/All%20languages%20combined/meaning/{ch1}/{part2}/{lower_word}.jsonl"
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