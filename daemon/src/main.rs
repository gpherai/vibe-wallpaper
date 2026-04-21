use anyhow::Context;
use log::{error, info, warn};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, Mutex};
use zbus::{connection::Builder, interface};

use vibe_core::compositor::Compositor;
use vibe_core::config::{AppConfig, QuoteProviderType, WallpaperProviderType};
use vibe_core::desktop::get_current_desktop;
use vibe_core::providers::quote::{LocalFileQuoteProvider, ZenQuotesProvider};
use vibe_core::providers::wallpaper::{EarthViewProvider, RedditProvider, UnsplashProvider};
use vibe_core::providers::{QuoteProvider, WallpaperProvider};

fn get_wallpaper_provider(
    config: &AppConfig,
) -> anyhow::Result<Box<dyn WallpaperProvider + Send + Sync>> {
    match config.provider_type {
        WallpaperProviderType::Reddit => {
            let subreddit = config.subreddit.trim();
            if subreddit.is_empty() {
                anyhow::bail!("subreddit cannot be empty when provider_type=reddit");
            }
            info!("Daemon: Initializing Reddit provider for r/{}", subreddit);
            Ok(Box::new(RedditProvider::new(subreddit)))
        }
        WallpaperProviderType::Unsplash => {
            let key = config
                .unsplash_access_key
                .as_ref()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            let key = key.context("unsplash_access_key is required when provider_type=unsplash")?;
            let query = config
                .unsplash_query
                .as_ref()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());

            info!(
                "Daemon: Initializing Unsplash provider (Query: {:?})",
                query
            );
            Ok(Box::new(UnsplashProvider::new(Some(key), query)))
        }
        WallpaperProviderType::EarthView => {
            info!("Daemon: Initializing Google Earth View provider");
            Ok(Box::new(EarthViewProvider::new()))
        }
    }
}

fn get_quote_provider(config: &AppConfig) -> anyhow::Result<Box<dyn QuoteProvider + Send + Sync>> {
    match config.quote_provider_type {
        QuoteProviderType::ZenQuotes => Ok(Box::new(ZenQuotesProvider)),
        QuoteProviderType::LocalFile => {
            let path = config
                .quote_local_path
                .clone()
                .context("quote_local_path is required when quote_provider_type=localfile")?;
            if path.to_string_lossy().trim().is_empty() {
                anyhow::bail!("quote_local_path is empty");
            }
            Ok(Box::new(LocalFileQuoteProvider::new(path)))
        }
    }
}

#[derive(Debug, Clone)]
struct DaemonState {
    paused: bool,
}

enum Command {
    Next,
    Pause,
    Resume,
    ReloadConfig,
}

struct VibeServer {
    tx: mpsc::Sender<Command>,
    state: Arc<Mutex<DaemonState>>,
    config: Arc<Mutex<AppConfig>>,
}

impl VibeServer {
    async fn send_command(&self, command: Command) -> zbus::fdo::Result<()> {
        self.tx.send(command).await.map_err(|err| {
            zbus::fdo::Error::Failed(format!("Daemon command channel unavailable: {}", err))
        })
    }
}

#[interface(name = "org.vibe.Daemon")]
impl VibeServer {
    async fn next(&self) -> zbus::fdo::Result<()> {
        info!("IPC: Received NEXT command");
        self.send_command(Command::Next).await
    }

    async fn pause(&self) -> zbus::fdo::Result<()> {
        info!("IPC: Received PAUSE command");
        {
            let mut state = self.state.lock().await;
            state.paused = true;
        }
        {
            let mut config = self.config.lock().await;
            config.is_paused = true;
            if let Err(e) = config.save() {
                error!("Daemon: Failed to save config: {}", e);
            }
        }
        self.send_command(Command::Pause).await
    }

    async fn resume(&self) -> zbus::fdo::Result<()> {
        info!("IPC: Received RESUME command");
        {
            let mut state = self.state.lock().await;
            state.paused = false;
        }
        {
            let mut config = self.config.lock().await;
            config.is_paused = false;
            if let Err(e) = config.save() {
                error!("Daemon: Failed to save config: {}", e);
            }
        }
        self.send_command(Command::Resume).await
    }

    async fn reload_config(&self) -> zbus::fdo::Result<()> {
        info!("IPC: Received RELOAD_CONFIG command");
        let loaded = AppConfig::load_strict()
            .map_err(|err| zbus::fdo::Error::Failed(format!("Failed to load config: {}", err)))?;
        if let Err(err) = loaded.validate() {
            return Err(zbus::fdo::Error::Failed(format!(
                "Reloaded config is invalid: {}",
                err
            )));
        }

        let paused = loaded.is_paused;
        {
            let mut config = self.config.lock().await;
            *config = loaded;
        }
        {
            let mut state = self.state.lock().await;
            state.paused = paused;
        }

        self.send_command(Command::ReloadConfig).await
    }

    async fn status(&self) -> zbus::fdo::Result<String> {
        let state = self.state.lock().await;
        if state.paused {
            Ok("Paused".to_string())
        } else {
            Ok("Running".to_string())
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    info!("Vibe Daemon starting up...");

    let mut loaded_config = AppConfig::load();
    if let Err(err) = loaded_config.validate() {
        error!(
            "Daemon: Invalid config detected ({}). Falling back to defaults.",
            err
        );
        loaded_config = AppConfig::default();
        if let Err(save_err) = loaded_config.save() {
            warn!(
                "Daemon: Failed to persist default config after invalid load: {}",
                save_err
            );
        }
    }

    let is_paused = loaded_config.is_paused;
    let config = Arc::new(Mutex::new(loaded_config));

    let state = Arc::new(Mutex::new(DaemonState { paused: is_paused }));
    let (tx, mut rx) = mpsc::channel(32);

    let server = VibeServer {
        tx,
        state: state.clone(),
        config: config.clone(),
    };

    let _connection = Builder::session()?
        .name("org.vibe.Daemon")?
        .serve_at("/org/vibe/Daemon", server)?
        .build()
        .await?;

    info!("D-Bus interface registered: org.vibe.Daemon");

    let desktop = get_current_desktop();

    let mut output_dir = std::env::temp_dir();
    output_dir.push("vibe");
    if let Err(e) = std::fs::create_dir_all(&output_dir) {
        error!("Daemon: Failed to create /tmp/vibe directory: {}", e);
    }

    let mut cached_quote: Option<String> = None;
    let mut last_quote_refresh: Option<Instant> = None;
    let mut should_fetch = !is_paused;

    loop {
        if should_fetch {
            info!("Cycle: Triggering wallpaper update...");

            let conf = { config.lock().await.clone() };
            let cycle_result = (|| -> anyhow::Result<_> {
                let wallpaper_provider = get_wallpaper_provider(&conf)?;
                let quote_provider = get_quote_provider(&conf)?;
                let quote_interval =
                    Duration::from_secs(u64::from(conf.quote_interval_mins.max(1)) * 60);
                Ok((wallpaper_provider, quote_provider, quote_interval))
            })();

            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let output_path = output_dir.join(format!("wallpaper_{}.jpg", timestamp));

            let update_result = match cycle_result {
                Ok((wallpaper_provider, quote_provider, quote_interval)) => {
                    run_cycle(
                        wallpaper_provider.as_ref(),
                        quote_provider.as_ref(),
                        &*desktop,
                        &output_path,
                        &mut cached_quote,
                        &mut last_quote_refresh,
                        quote_interval,
                    )
                    .await
                }
                Err(err) => Err(err),
            };

            match update_result {
                Ok(_) => {
                    info!("Cycle: Successfully updated wallpaper.");
                    if let Ok(entries) = std::fs::read_dir(&output_dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_file() && path != output_path {
                                let _ = std::fs::remove_file(path);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Cycle: Failed to update wallpaper: {:#}", e);
                    info!("Cycle: Retrying in 2 minutes...");
                    tokio::select! {
                        _ = tokio::time::sleep(Duration::from_secs(2 * 60)) => {
                            should_fetch = true;
                            continue;
                        }
                        cmd = rx.recv() => {
                            match cmd {
                                Some(Command::Next) => should_fetch = true,
                                Some(Command::Pause) => should_fetch = false,
                                Some(Command::Resume) => should_fetch = true,
                                Some(Command::ReloadConfig) => {
                                    cached_quote = None;
                                    last_quote_refresh = None;
                                    let st = state.lock().await;
                                    should_fetch = !st.paused;
                                }
                                None => break,
                            }
                            continue;
                        }
                    }
                }
            }
        }

        should_fetch = false;

        let interval_mins = {
            let conf = config.lock().await;
            conf.wallpaper_interval_mins.max(1)
        };

        info!(
            "Status: Waiting for next interval ({} mins)...",
            interval_mins
        );
        let sleep_duration = Duration::from_secs(u64::from(interval_mins) * 60);

        tokio::select! {
            _ = tokio::time::sleep(sleep_duration) => {
                let st = state.lock().await;
                if !st.paused {
                    should_fetch = true;
                }
            }
            cmd = rx.recv() => {
                match cmd {
                    Some(Command::Next) => {
                        info!("Event: NEXT command triggered fetch.");
                        should_fetch = true;
                    }
                    Some(Command::Pause) => {
                        info!("Event: PAUSE command received.");
                        should_fetch = false;
                    }
                    Some(Command::Resume) => {
                        info!("Event: RESUME command received.");
                        should_fetch = true;
                    }
                    Some(Command::ReloadConfig) => {
                        info!("Event: RELOAD_CONFIG command received.");
                        cached_quote = None;
                        last_quote_refresh = None;
                        let st = state.lock().await;
                        should_fetch = !st.paused;
                    }
                    None => break,
                }
            }
        }
    }

    Ok(())
}

async fn run_cycle(
    wallpaper_provider: &(dyn WallpaperProvider + Send + Sync),
    quote_provider: &(dyn QuoteProvider + Send + Sync),
    desktop: &dyn vibe_core::desktop::DesktopEnvironment,
    output_path: &std::path::Path,
    cached_quote: &mut Option<String>,
    last_quote_refresh: &mut Option<Instant>,
    quote_interval: Duration,
) -> anyhow::Result<()> {
    let quote_expired = last_quote_refresh
        .map(|instant| instant.elapsed() >= quote_interval)
        .unwrap_or(true);

    let (wallpaper_bytes, quote) = if quote_expired {
        let (wallpaper_result, quote_result) = tokio::join!(
            wallpaper_provider.fetch_wallpaper(),
            quote_provider.fetch_quote()
        );

        let wallpaper_bytes = wallpaper_result?;
        let quote = match quote_result {
            Ok(new_quote) => {
                *cached_quote = Some(new_quote.clone());
                *last_quote_refresh = Some(Instant::now());
                new_quote
            }
            Err(err) => {
                if let Some(existing_quote) = cached_quote.clone() {
                    warn!(
                        "Cycle: Quote refresh failed ({}). Reusing last successful quote.",
                        err
                    );
                    existing_quote
                } else {
                    return Err(err);
                }
            }
        };

        (wallpaper_bytes, quote)
    } else {
        let wallpaper_bytes = wallpaper_provider.fetch_wallpaper().await?;
        let quote = cached_quote
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Quote cache is unexpectedly empty"))?;
        (wallpaper_bytes, quote)
    };

    info!("Compositor: Applying quote to wallpaper...");
    Compositor::process_and_save(&wallpaper_bytes, &quote, output_path).await?;

    info!("Desktop: Setting wallpaper file: {:?}", output_path);
    desktop.set_wallpaper(output_path).await?;

    Ok(())
}
