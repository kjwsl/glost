use clap::Parser;
use glost::{cli::Args, commands::handle_command};

slint::include_modules!();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::new()?.run()?;

    Ok(())
}
