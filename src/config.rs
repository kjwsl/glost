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

/// Migrate the filter file from the current directory to the config directory
pub fn migrate_filter_file_if_needed() -> Result<(), std::io::Error> {
    let old_path = PathBuf::from("filter.txt");
    let new_path = get_config_dir().join("filter.txt");
    
    // If the old file exists and the new one doesn't, migrate it
    if old_path.exists() && !new_path.exists() {
        std::fs::copy(&old_path, &new_path)?;
        println!("Migrated filter.txt to config directory: {}", new_path.display());
        
        // Optionally remove the old file (commented out for safety)
        // std::fs::remove_file(&old_path)?;
    }
    
    Ok(())
}