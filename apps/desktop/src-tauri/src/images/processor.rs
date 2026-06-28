//! Image processing utilities — resize, convert, optimize.

use image::{DynamicImage, GenericImageView, ImageFormat};
use std::path::{Path, PathBuf};

/// Resize an image to target dimensions while maintaining aspect ratio
pub fn resize_image(
    input_path: &Path,
    output_path: &Path,
    target_width: u32,
    target_height: u32,
) -> Result<(), String> {
    // Load image
    let img = image::open(input_path).map_err(|e| format!("Failed to open image: {}", e))?;

    // Resize with aspect ratio preservation
    let resized = resize_with_aspect_ratio(&img, target_width, target_height);

    // Determine output format from extension
    let format = match output_path.extension().and_then(|s| s.to_str()) {
        Some("jpg") | Some("jpeg") => ImageFormat::Jpeg,
        Some("png") => ImageFormat::Png,
        Some("webp") => ImageFormat::WebP,
        _ => ImageFormat::Jpeg, // Default to JPEG
    };

    // Save
    resized
        .save_with_format(output_path, format)
        .map_err(|e| format!("Failed to save image: {}", e))?;

    Ok(())
}

/// Resize with aspect ratio preservation (fit within bounds)
fn resize_with_aspect_ratio(
    img: &DynamicImage,
    target_width: u32,
    target_height: u32,
) -> DynamicImage {
    let (orig_width, orig_height) = img.dimensions();

    // Calculate scale to fit within target dimensions
    let width_scale = target_width as f32 / orig_width as f32;
    let height_scale = target_height as f32 / orig_height as f32;
    let scale = width_scale.min(height_scale);

    let new_width = (orig_width as f32 * scale) as u32;
    let new_height = (orig_height as f32 * scale) as u32;

    img.resize(
        new_width,
        new_height,
        image::imageops::FilterType::Lanczos3,
    )
}

/// Process and optimize image from download
pub fn process_downloaded_image(
    input_path: &Path,
    output_path: &Path,
    target_width: u32,
    target_height: u32,
) -> Result<PathBuf, String> {
    resize_image(input_path, output_path, target_width, target_height)?;
    Ok(output_path.to_path_buf())
}

/// Convert image format
pub fn convert_format(
    input_path: &Path,
    output_path: &Path,
    format: ImageFormat,
) -> Result<(), String> {
    let img = image::open(input_path).map_err(|e| format!("Failed to open image: {}", e))?;
    
    img.save_with_format(output_path, format)
        .map_err(|e| format!("Failed to convert image: {}", e))?;

    Ok(())
}
