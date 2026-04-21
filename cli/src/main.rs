use clap::{Parser, Subcommand};
use vibe_core::ipc::VibeControlProxy;
use zbus::Connection;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Skip to the next wallpaper
    Next,
    /// Pause wallpaper rotation
    Pause,
    /// Resume wallpaper rotation
    Resume,
    /// Reload configuration
    Reload,
    /// Get current status
    Status,
    /// Save current wallpaper as favorite
    Favorite,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let conn = Connection::session().await?;
    let proxy = VibeControlProxy::new(&conn).await?;

    match cli.command {
        Commands::Next => {
            proxy.next().await?;
            println!("Skipped to next wallpaper.");
        }
        Commands::Pause => {
            proxy.pause().await?;
            println!("Rotation paused.");
        }
        Commands::Resume => {
            proxy.resume().await?;
            println!("Rotation resumed.");
        }
        Commands::Reload => {
            proxy.reload_config().await?;
            println!("Configuration reloaded.");
        }
        Commands::Status => {
            let status = proxy.status().await?;
            println!("Status: {}", status);
        }
        Commands::Favorite => {
            proxy.favorite().await?;
            println!("Saved current wallpaper to favorites.");
        }
    }

    Ok(())
}
