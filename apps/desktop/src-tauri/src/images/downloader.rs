//! Async image downloader with retry logic.

use std::path::PathBuf;
use std::time::Duration;

/// Maximum retry attempts for failed downloads
const MAX_RETRIES: u32 = 3;

/// Download result
#[derive(Debug)]
pub struct DownloadResult {
    pub local_path: PathBuf,
    pub size_bytes: u64,
}

/// Download an image from a URL with retry logic
pub async fn download_image(
    url: &str,
    output_path: PathBuf,
) -> Result<DownloadResult, String> {
    let mut attempts = 0;

    loop {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| e.to_string())?;

        match client.get(url).send().await {
            Ok(response) => {
                if !response.status().is_success() {
                    return Err(format!("HTTP error: {}", response.status()));
                }

                let bytes = response.bytes().await.map_err(|e| e.to_string())?;
                
                // Write to file
                tokio::fs::write(&output_path, &bytes)
                    .await
                    .map_err(|e| format!("Failed to write image: {}", e))?;

                let size_bytes = bytes.len() as u64;

                return Ok(DownloadResult {
                    local_path: output_path,
                    size_bytes,
                });
            }
            Err(e) => {
                attempts += 1;
                if attempts >= MAX_RETRIES {
                    return Err(format!("Download failed after {} retries: {}", MAX_RETRIES, e));
                }
                
                // Exponential backoff
                let delay = Duration::from_millis(1000 * 2_u64.pow(attempts));
                tokio::time::sleep(delay).await;
            }
        }
    }
}

/// Download multiple images concurrently
pub async fn download_images_batch(
    downloads: Vec<(String, PathBuf)>,
) -> Vec<Result<DownloadResult, String>> {
    let mut tasks = Vec::new();

    for (url, path) in downloads {
        let task = tokio::spawn(async move {
            download_image(&url, path).await
        });
        tasks.push(task);
    }

    let mut results = Vec::new();
    for task in tasks {
        match task.await {
            Ok(result) => results.push(result),
            Err(e) => results.push(Err(format!("Task failed: {}", e))),
        }
    }

    results
}
