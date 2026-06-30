//! Image processing and management utilities for T20.
//!
//! Handles downloading, resizing, and storing game cover art and backgrounds
//! from metadata API URLs.
//!
//! T25: Some image utilities (batch download, thumbnail) deferred to polish phase.
#![allow(dead_code)]


pub mod downloader;
pub mod processor;

use std::path::PathBuf;

/// Image storage paths
pub struct ImagePaths {
    pub covers: PathBuf,
    pub backgrounds: PathBuf,
    pub thumbnails: PathBuf,
}

impl ImagePaths {
    /// Initialize image storage directories
    pub fn new(app_data_dir: &std::path::Path) -> Result<Self, String> {
        let images_dir = app_data_dir.join("images");
        
        let paths = ImagePaths {
            covers: images_dir.join("covers"),
            backgrounds: images_dir.join("backgrounds"),
            thumbnails: images_dir.join("thumbnails"),
        };

        // Create directories if they don't exist
        std::fs::create_dir_all(&paths.covers).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&paths.backgrounds).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&paths.thumbnails).map_err(|e| e.to_string())?;

        Ok(paths)
    }
}

/// Image type for downloads
#[derive(Debug, Clone, Copy)]
pub enum ImageType {
    Cover,
    Background,
    Thumbnail,
}

impl ImageType {
    pub fn target_dimensions(&self) -> (u32, u32) {
        match self {
            ImageType::Cover => (512, 512),
            ImageType::Background => (1920, 1080),
            ImageType::Thumbnail => (256, 256),
        }
    }

    pub fn filename_suffix(&self) -> &'static str {
        match self {
            ImageType::Cover => "cover",
            ImageType::Background => "background",
            ImageType::Thumbnail => "thumbnail",
        }
    }
}
