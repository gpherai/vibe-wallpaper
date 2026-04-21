use anyhow::Context;
use directories::ProjectDirs;
use log::warn;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WallpaperProviderType {
    #[default]
    Reddit,
    Unsplash,
    EarthView,
    Bing,
    Wallhaven,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum QuoteProviderType {
    #[default]
    ZenQuotes,
    LocalFile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub wallpaper_interval_mins: u32,
    pub quote_interval_mins: u32,
    pub subreddit: String,
    pub provider_type: WallpaperProviderType,
    pub unsplash_access_key: Option<String>,
    pub unsplash_query: Option<String>,
    pub quote_provider_type: QuoteProviderType,
    pub quote_local_path: Option<PathBuf>,
    pub is_paused: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            wallpaper_interval_mins: 60,
            quote_interval_mins: 60,
            subreddit: "EarthPorn".to_string(),
            provider_type: WallpaperProviderType::Reddit,
            unsplash_access_key: None,
            unsplash_query: Some("nature,wallpapers".to_string()),
            quote_provider_type: QuoteProviderType::ZenQuotes,
            quote_local_path: None,
            is_paused: false,
        }
    }
}

impl AppConfig {
    pub fn config_dir() -> Option<PathBuf> {
        ProjectDirs::from("com", "vibe", "vibe").map(|dirs| dirs.config_dir().to_path_buf())
    }

    pub fn config_path() -> Option<PathBuf> {
        Self::config_dir().map(|dir| dir.join("config.toml"))
    }

    pub fn load_strict() -> anyhow::Result<Self> {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                let content = fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read config at {}", path.display()))?;
                let config = toml::from_str(&content)
                    .with_context(|| format!("Failed to parse config at {}", path.display()))?;
                return Ok(config);
            }
        }
        Ok(Self::default())
    }

    pub fn load() -> Self {
        match Self::load_strict() {
            Ok(config) => config,
            Err(err) => {
                warn!("{}. Falling back to defaults.", err);
                Self::default()
            }
        }
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.wallpaper_interval_mins == 0 {
            anyhow::bail!("wallpaper_interval_mins must be greater than 0");
        }
        if self.quote_interval_mins == 0 {
            anyhow::bail!("quote_interval_mins must be greater than 0");
        }

        if matches!(self.provider_type, WallpaperProviderType::Reddit)
            && self.subreddit.trim().is_empty()
        {
            anyhow::bail!("subreddit cannot be empty when provider_type=reddit");
        }

        if matches!(self.provider_type, WallpaperProviderType::Unsplash)
            && self
                .unsplash_access_key
                .as_ref()
                .map(|key| key.trim().is_empty())
                .unwrap_or(true)
        {
            anyhow::bail!("unsplash_access_key is required when provider_type=unsplash");
        }

        if matches!(self.quote_provider_type, QuoteProviderType::LocalFile)
            && self
                .quote_local_path
                .as_ref()
                .map(|p| p.to_string_lossy().trim().is_empty())
                .unwrap_or(true)
        {
            anyhow::bail!("quote_local_path is required when quote_provider_type=localfile");
        }

        Ok(())
    }

    pub fn save(&self) -> anyhow::Result<()> {
        if let Some(dir) = Self::config_dir() {
            fs::create_dir_all(&dir)?;
            let path = dir.join("config.toml");
            let tmp_path = dir.join("config.toml.tmp");
            let content = toml::to_string_pretty(self)?;
            fs::write(&tmp_path, content)?;
            fs::rename(&tmp_path, &path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{AppConfig, QuoteProviderType, WallpaperProviderType};

    #[test]
    fn deserialize_uses_defaults_for_missing_values() {
        let config: AppConfig = toml::from_str("").expect("empty config should deserialize");
        assert_eq!(config.provider_type, WallpaperProviderType::Reddit);
        assert_eq!(config.quote_provider_type, QuoteProviderType::ZenQuotes);
        assert_eq!(config.wallpaper_interval_mins, 60);
        assert_eq!(config.quote_interval_mins, 60);
        assert_eq!(config.subreddit, "EarthPorn");
    }

    #[test]
    fn validate_rejects_invalid_intervals() {
        let config = AppConfig {
            wallpaper_interval_mins: 0,
            ..AppConfig::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn validate_requires_local_quote_path_for_localfile_provider() {
        let config = AppConfig {
            quote_provider_type: QuoteProviderType::LocalFile,
            quote_local_path: None,
            ..AppConfig::default()
        };
        assert!(config.validate().is_err());
    }
}
