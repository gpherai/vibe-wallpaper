use image::{DynamicImage, RgbaImage};

fn test() {
    let rgba = RgbaImage::new(10, 10);
    let rgb = DynamicImage::ImageRgba8(rgba).into_rgb8();
}
