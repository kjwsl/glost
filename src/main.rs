use clap::Parser;
use glost::{cli::Args, commands::handle_command};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    handle_command(args.command).await?;
    Ok(())
}
