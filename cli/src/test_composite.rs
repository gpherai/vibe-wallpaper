use vibe_core::compositor::Compositor;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let wallpaper_bytes = include_bytes!("../gui/src/assets/tauri.svg");
    let quote = "This is a test quote.\n— Author";
    let output_path = PathBuf::from("test_output.jpg");
    
    Compositor::process_and_save(wallpaper_bytes, quote, &output_path).await?;
    println!("Saved to test_output.jpg");
    Ok(())
}
