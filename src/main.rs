use clap::Parser;
use glost::{cli::Args, commands::handle_command, config::migrate_filter_file_if_needed};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Migrate filter file from current directory to config directory if needed
    if let Err(e) = migrate_filter_file_if_needed() {
        eprintln!("Warning: Failed to migrate filter file: {}", e);
    }
    
    let args = Args::parse();
    handle_command(args.command).await
}
