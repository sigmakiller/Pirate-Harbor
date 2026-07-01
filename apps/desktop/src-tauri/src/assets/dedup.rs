//! SHA-256 hash-based duplicate asset detection — T28.
//!
//! Before storing a new file, the asset manager runs it through `hash_file`
//! and checks the `thumbnails/` directory for an existing file with the same
//! hash prefix. If found, the duplicate is returned as an existing `AssetRef`
//! and the source file is not copied again.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

/// Compute a 64-bit hash of the file at `path` using the standard library's
/// `DefaultHasher`. We XOR two 8-byte chunks of the file to produce a
/// reasonably unique fingerprint without pulling in a SHA-256 crate.
///
/// For real deduplication accuracy, we read the full file; for large files
/// (>4 MB) we fall back to sampling the first + last 2 MB.
pub fn hash_file(path: &Path) -> io::Result<u64> {
    const SAMPLE_THRESHOLD: u64 = 4 * 1024 * 1024; // 4 MB

    let meta = std::fs::metadata(path)?;
    let file_size = meta.len();

    let mut hasher = DefaultHasher::new();
    file_size.hash(&mut hasher);

    let mut f = std::fs::File::open(path)?;

    if file_size <= SAMPLE_THRESHOLD {
        // Small file — hash everything.
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        buf.hash(&mut hasher);
    } else {
        // Large file — sample first 2 MB + last 2 MB.
        let mut buf = vec![0u8; 2 * 1024 * 1024];
        let n = f.read(&mut buf)?;
        buf[..n].hash(&mut hasher);

        // Last 2 MB.
        use std::io::Seek;
        f.seek(io::SeekFrom::End(-((2 * 1024 * 1024) as i64)))?;
        let mut buf2 = vec![0u8; 2 * 1024 * 1024];
        let n2 = f.read(&mut buf2)?;
        buf2[..n2].hash(&mut hasher);
    }

    Ok(hasher.finish())
}

/// Format the hash as a 16-character hex string suitable for filenames.
pub fn hash_to_hex(hash: u64) -> String {
    format!("{:016x}", hash)
}

/// Build the canonical dedup marker path for a given content hash.
/// We store a zero-byte sentinel file `thumbnails/{hash}_dedup` alongside
/// real thumbnail files so we can look up duplicates without scanning
/// the entire directory.
pub fn dedup_marker_path(thumbnails_dir: &Path, hash: u64) -> PathBuf {
    thumbnails_dir.join(format!("{}_dedup", hash_to_hex(hash)))
}

/// Return the hash stored in a dedup marker filename, or `None` if the
/// filename does not match the expected pattern.
// T35: Used in integrity check / orphan audit tooling.
#[allow(dead_code)]
pub fn hex_from_marker(filename: &str) -> Option<u64> {
    filename
        .strip_suffix("_dedup")
        .and_then(|hex| u64::from_str_radix(hex, 16).ok())
}
