use std::path::PathBuf;

/// Get the default filter file path in the config directory
pub fn default_filter_file_path() -> String {
    get_config_dir()
        .join("filter.txt")
        .to_string_lossy()
        .to_string()
}

/// Get the default cache file path in the data directory
pub fn default_cache_file_path() -> String {
    get_data_dir()
        .join("cache.db")
        .to_string_lossy()
        .to_string()
}

/// Get the glost config directory, creating it if it doesn't exist
pub fn get_config_dir() -> PathBuf {
    let home_dir = dirs::home_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let config_dir = home_dir.join(".config").join("glost");

    // Create the config directory if it doesn't exist
    if !config_dir.exists() {
        let _ = std::fs::create_dir_all(&config_dir);
    }

    config_dir
}

/// Get the glost data directory, creating it if it doesn't exist
pub fn get_data_dir() -> PathBuf {
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| {
            dirs::home_dir()
                .map(|h| h.join(".local").join("share"))
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        })
        .join("glost");

    // Create the data directory if it doesn't exist
    if !data_dir.exists() {
        let _ = std::fs::create_dir_all(&data_dir);
    }

    data_dir
}
