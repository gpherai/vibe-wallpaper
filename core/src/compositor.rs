use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use anyhow::Context;
use image::Rgba;
use imageproc::drawing::draw_text_mut;
use log::info;
use std::path::{Path, PathBuf};

pub struct Compositor;

impl Compositor {
    fn find_font() -> Option<PathBuf> {
        let common_paths = [
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/dejavu-sans-fonts/DejaVuSans.ttf",
            "/usr/share/fonts/liberation/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
            "/usr/share/fonts/liberation-sans/LiberationSans-Regular.ttf",
        ];

        for path in common_paths {
            let p = PathBuf::from(path);
            if p.exists() {
                return Some(p);
            }
        }

        // Fallback to fc-match if available
        if let Ok(output) = std::process::Command::new("fc-match")
            .arg("-f")
            .arg("%{file}")
            .arg("sans-serif")
            .output()
        {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let p = PathBuf::from(path_str);
                if p.exists() {
                    return Some(p);
                }
            }
        }

        None
    }

    pub async fn process_and_save(
        wallpaper_bytes: &[u8],
        quote_text: &str,
        output_path: &Path,
    ) -> crate::Result<()> {
        let image =
            image::load_from_memory(wallpaper_bytes).context("Failed to decode wallpaper image")?;
        
        let mut image = image.to_rgba8();

        if let Some(font_path) = Self::find_font() {
            let font_data = std::fs::read(font_path).context("Failed to read font file")?;
            let font = FontRef::try_from_slice(&font_data).context("Failed to parse font")?;

            let img_width = image.width() as f32;
            let img_height = image.height() as f32;
            let shortest_side = img_width.min(img_height);

            // Modern typography settings
            let font_size = (img_height / 25.0).clamp(18.0, 72.0); // Responsive font size
            let scale = PxScale::from(font_size);
            let padding = (shortest_side * 0.05).clamp(16.0, 60.0);
            let max_text_width = (img_width - (padding * 2.0)).max(48.0);

            // Simple word wrap
            let wrapped_lines = Self::wrap_text(quote_text, &font, scale, max_text_width);

            let line_height = font_size * 1.2;
            let total_text_height = wrapped_lines.len() as f32 * line_height;

            // Position: Centered at the bottom third
            let mut y = (img_height - total_text_height - padding).max(padding / 2.0);
            let text_color = Rgba([255u8, 255, 255, 255]);
            let shadow_color = Rgba([0u8, 0, 0, 180]);

            for line in wrapped_lines {
                // Center align horizontally
                let line_width = Self::get_text_width(&line, &font, scale);
                let max_x = (img_width - line_width).max(0.0);
                let x = ((img_width - line_width) / 2.0).clamp(0.0, max_x);

                // Draw shadow for depth
                draw_text_mut(
                    &mut image,
                    shadow_color,
                    (x + 3.0) as i32,
                    (y + 3.0) as i32,
                    scale,
                    &font,
                    &line,
                );
                // Draw main text
                draw_text_mut(
                    &mut image, text_color, x as i32, y as i32, scale, &font, &line,
                );

                y += line_height;
            }
        } else {
            info!("No suitable font found, saving image without text.");
        }

        let rgb_image = image::DynamicImage::ImageRgba8(image).into_rgb8();

        rgb_image
            .save(output_path)
            .context("Failed to save composite image")?;
        Ok(())
    }

    fn wrap_text(text: &str, font: &FontRef, scale: PxScale, max_width: f32) -> Vec<String> {
        let mut lines = Vec::new();
        for paragraph in text.split('\n') {
            let mut current_line = String::new();
            for word in paragraph.split_whitespace() {
                let test_line = if current_line.is_empty() {
                    word.to_string()
                } else {
                    format!("{} {}", current_line, word)
                };

                if Self::get_text_width(&test_line, font, scale) > max_width {
                    if !current_line.is_empty() {
                        lines.push(current_line);
                    }
                    current_line = word.to_string();
                } else {
                    current_line = test_line;
                }
            }
            if !current_line.is_empty() {
                lines.push(current_line);
            }
        }
        lines
    }

    fn get_text_width(text: &str, font: &FontRef, scale: PxScale) -> f32 {
        let scaled_font = font.as_scaled(scale);
        let mut width = 0.0;
        let mut last_glyph_id = None;
        for c in text.chars() {
            let glyph_id = font.glyph_id(c);
            width += scaled_font.h_advance(glyph_id);
            if let Some(last) = last_glyph_id {
                width += scaled_font.kern(last, glyph_id);
            }
            last_glyph_id = Some(glyph_id);
        }
        width
    }
}
