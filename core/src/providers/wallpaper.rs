use super::WallpaperProvider;
use anyhow::Context;
use log::info;
use serde::Deserialize;

pub struct RedditProvider {
    pub subreddit: String,
}

#[derive(Deserialize)]
struct RedditResponse {
    data: RedditData,
}

#[derive(Deserialize)]
struct RedditData {
    children: Vec<RedditChild>,
}

#[derive(Deserialize)]
struct RedditChild {
    data: PostData,
}

#[derive(Deserialize)]
struct PostData {
    url: String,
    #[serde(default)]
    is_video: bool,
    preview: Option<RedditPreview>,
}

#[derive(Deserialize)]
struct RedditPreview {
    images: Vec<RedditImage>,
}

#[derive(Deserialize)]
struct RedditImage {
    source: RedditImageSource,
}

#[derive(Deserialize)]
struct RedditImageSource {
    url: String,
}

impl RedditProvider {
    pub fn new(subreddit: &str) -> Self {
        Self {
            subreddit: subreddit.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl WallpaperProvider for RedditProvider {
    async fn fetch_wallpaper(&self) -> crate::Result<Vec<u8>> {
        let url = format!(
            "https://www.reddit.com/r/{}/hot.json?limit=30",
            self.subreddit
        );
        info!("Reddit: Fetching hot posts from r/{}", self.subreddit);

        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 VibeApp/1.0")
            .timeout(std::time::Duration::from_secs(20))
            .build()?;

        let response = client
            .get(&url)
            .send()
            .await
            .context("Reddit: Failed to send request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Reddit API error: {} - {}", status, body);
        }

        let resp: RedditResponse = response
            .json()
            .await
            .context("Reddit: Failed to parse JSON response. Check if the subreddit exists.")?;

        info!(
            "Reddit: Parsing {} children for images...",
            resp.data.children.len()
        );

        let image_url = resp
            .data
            .children
            .into_iter()
            .map(|child| child.data)
            .find_map(|data| {
                let lower_url = data.url.to_lowercase();
                // Check if it's a direct image link
                if !data.is_video
                    && (lower_url.ends_with(".jpg")
                        || lower_url.ends_with(".jpeg")
                        || lower_url.ends_with(".png"))
                {
                    Some(data.url)
                } else if let Some(preview) = data.preview {
                    // Check if it has a preview image (often used for galleries/external links)
                    preview
                        .images
                        .first()
                        .map(|img| img.source.url.replace("&amp;", "&"))
                } else {
                    None
                }
            })
            .context(format!(
                "Reddit: No suitable images found in the top 30 posts of r/{}",
                self.subreddit
            ))?;

        info!("Reddit: Selected image URL: {}", image_url);
        let image_response = client
            .get(&image_url)
            .send()
            .await
            .context("Reddit: Failed to download selected image")?
            .error_for_status()
            .context("Reddit: Selected image URL returned a non-success status")?;

        let image_bytes = image_response
            .bytes()
            .await
            .context("Reddit: Failed to read image bytes")?;

        Ok(image_bytes.to_vec())
    }
}

pub struct UnsplashProvider {
    pub access_key: Option<String>,
    pub query: Option<String>,
}

#[derive(Deserialize, Debug)]
struct UnsplashResponse {
    urls: UnsplashUrls,
}

#[derive(Deserialize, Debug)]
struct UnsplashUrls {
    full: String,
}

impl UnsplashProvider {
    pub fn new(access_key: Option<String>, query: Option<String>) -> Self {
        Self { access_key, query }
    }

    fn build_random_photo_url(query: Option<&str>) -> crate::Result<reqwest::Url> {
        let mut url = reqwest::Url::parse("https://api.unsplash.com/photos/random")
            .context("Unsplash: Failed to build API URL")?;

        if let Some(q) = query.filter(|q| !q.trim().is_empty()) {
            url.query_pairs_mut().append_pair("query", q);
        }

        Ok(url)
    }
}

#[async_trait::async_trait]
impl WallpaperProvider for UnsplashProvider {
    async fn fetch_wallpaper(&self) -> crate::Result<Vec<u8>> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(20))
            .build()?;

        let key = self
            .access_key
            .as_deref()
            .map(str::trim)
            .filter(|key| !key.is_empty())
            .context("Unsplash: Access Key is required when using Unsplash provider")?;
        let api_url = Self::build_random_photo_url(self.query.as_deref())?;

        info!(
            "Unsplash: Requesting random photo (Query: {:?})",
            self.query
        );
        let response = client
            .get(api_url)
            .header("Accept-Version", "v1")
            .header("Authorization", format!("Client-ID {}", key))
            .send()
            .await
            .context("Unsplash: Failed to send API request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Unsplash API error: {} - {}", status, body);
        }

        let resp: UnsplashResponse = response
            .json()
            .await
            .context("Unsplash: Failed to parse API JSON")?;
        let image_url = resp.urls.full;

        info!("Unsplash: Downloading image...");
        let image_response = client
            .get(&image_url)
            .send()
            .await
            .context("Unsplash: Failed to download image")?
            .error_for_status()
            .context("Unsplash: Image download returned a non-success status")?;
        let image_bytes = image_response.bytes().await?;
        Ok(image_bytes.to_vec())
    }
}

pub struct EarthViewProvider;

#[derive(Deserialize)]
struct EarthViewItem {
    image: String,
}

impl EarthViewProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EarthViewProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl WallpaperProvider for EarthViewProvider {
    async fn fetch_wallpaper(&self) -> crate::Result<Vec<u8>> {
        info!("EarthView: Fetching collection...");
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(25))
            .build()?;

        let response = client
            .get("https://raw.githubusercontent.com/limhenry/earthview/master/earthview.json")
            .send()
            .await
            .context("EarthView: Failed to fetch collection list")?;

        if !response.status().is_success() {
            anyhow::bail!("EarthView: HTTP Error {}", response.status());
        }

        let resp: Vec<EarthViewItem> = response
            .json()
            .await
            .context("EarthView: Failed to parse collection JSON")?;

        use rand::seq::IteratorRandom;
        let item = resp
            .into_iter()
            .choose(&mut rand::thread_rng())
            .context("EarthView: Collection is empty")?;

        info!("EarthView: Selected random image: {}", item.image);
        let image_response = client
            .get(&item.image)
            .send()
            .await
            .context("EarthView: Failed to download image")?
            .error_for_status()
            .context("EarthView: Selected image URL returned a non-success status")?;
        let image_bytes = image_response.bytes().await?;
        Ok(image_bytes.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::UnsplashProvider;

    #[test]
    fn build_unsplash_url_encodes_query_parameters() {
        let url = UnsplashProvider::build_random_photo_url(Some("nature wallpaper,city"))
            .expect("url build should succeed");
        let rendered = url.as_str();
        assert!(rendered.contains("query=nature+wallpaper%2Ccity"));
    }
}
