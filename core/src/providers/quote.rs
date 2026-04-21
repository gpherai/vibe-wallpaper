use super::QuoteProvider;
use anyhow::bail;
use anyhow::Context;
use rand::seq::SliceRandom;
use serde::Deserialize;
use std::path::PathBuf;

pub struct ZenQuotesProvider;

#[derive(Deserialize)]
struct ZenQuote {
    q: String, // Quote
    a: String, // Author
}

#[async_trait::async_trait]
impl QuoteProvider for ZenQuotesProvider {
    async fn fetch_quote(&self) -> crate::Result<String> {
        let url = "https://zenquotes.io/api/random";

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(20))
            .build()?;
        let response = client
            .get(url)
            .send()
            .await
            .context("ZenQuotes: Failed to send API request")?
            .error_for_status()
            .context("ZenQuotes: API returned a non-success status")?;

        let quotes: Vec<ZenQuote> = response
            .json()
            .await
            .context("ZenQuotes: Failed to parse API response")?;

        let quote = quotes
            .into_iter()
            .next()
            .context("No quote received from API")?;

        Ok(format!("\"{}\"\n— {}", quote.q, quote.a))
    }
}

pub struct LocalFileQuoteProvider {
    pub path: PathBuf,
}

impl LocalFileQuoteProvider {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

#[async_trait::async_trait]
impl QuoteProvider for LocalFileQuoteProvider {
    async fn fetch_quote(&self) -> crate::Result<String> {
        if self.path.as_os_str().is_empty() {
            bail!("Local quote file path is empty");
        }
        if !self.path.exists() {
            bail!("Local quote file does not exist: {}", self.path.display());
        }

        let content = std::fs::read_to_string(&self.path)
            .context(format!("Failed to read local quote file: {:?}", self.path))?;

        let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
        let quote = lines
            .choose(&mut rand::thread_rng())
            .context("Quote file is empty")?;

        Ok(quote.to_string())
    }
}
