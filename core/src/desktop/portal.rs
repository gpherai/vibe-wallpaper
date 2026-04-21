use super::DesktopEnvironment;
use anyhow::Context;
use ashpd::desktop::wallpaper::{SetOn, WallpaperRequest};
use log::{info, warn};
use std::os::fd::AsFd;
use std::path::Path;
use std::process::Command;

pub struct PortalAdapter;

fn run_gsettings(schema: &str, key: &str, value: &str) -> crate::Result<()> {
    let output = Command::new("gsettings")
        .args(["set", schema, key, value])
        .output()
        .context("Failed to execute gsettings command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "gsettings set {} {} failed (status: {:?}): {}",
            schema,
            key,
            output.status.code(),
            stderr.trim()
        );
    }

    Ok(())
}

impl DesktopEnvironment for PortalAdapter {
    fn set_wallpaper(
        &self,
        path: &Path,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::Result<()>> + Send + '_>> {
        let path_owned = path.to_path_buf();
        Box::pin(async move {
            let file = std::fs::File::open(&path_owned)?;

            info!("Attempting to set wallpaper via XDG Portal...");
            let result = WallpaperRequest::default()
                .set_on(SetOn::Both)
                .show_preview(false)
                .build_file(&file.as_fd())
                .await;

            if let Err(e) = result {
                warn!(
                    "Portal method failed: {}. Trying GNOME gsettings fallback...",
                    e
                );

                let uri = reqwest::Url::from_file_path(&path_owned)
                    .map_err(|_| anyhow::anyhow!("Failed to convert wallpaper path to file URI"))?
                    .to_string();

                // Set for both light and dark mode
                run_gsettings("org.gnome.desktop.background", "picture-uri", &uri)
                    .context("gsettings fallback failed for picture-uri")?;
                run_gsettings("org.gnome.desktop.background", "picture-uri-dark", &uri)
                    .context("gsettings fallback failed for picture-uri-dark")?;

                info!("Wallpaper set via gsettings fallback.");
            } else {
                info!("Wallpaper successfully set via XDG Portal.");
            }

            Ok(())
        })
    }
}
