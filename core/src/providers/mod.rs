pub mod quote;
pub mod wallpaper;

#[async_trait::async_trait]
pub trait WallpaperProvider {
    async fn fetch_wallpaper(&self) -> crate::Result<Vec<u8>>;
}

#[async_trait::async_trait]
pub trait QuoteProvider {
    async fn fetch_quote(&self) -> crate::Result<String>;
}
