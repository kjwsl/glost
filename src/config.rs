use std::path::PathBuf;

/// Get the default filter file path in the config directory
pub fn default_filter_file_path() -> String {
    get_config_dir()
        .join("filter.txt")
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
