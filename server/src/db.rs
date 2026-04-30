/* Copyright (C) 2020 Yuval Deutscher

* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Database management: downloading, caching, parsing, and periodically
//! refreshing the GTFS feed from the Israel Ministry of Transport.

use futures_util::StreamExt;
use harail::RailroadData;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Duration;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Thread-safe handle to the live railroad database.
pub type SharedData = Arc<RwLock<RailroadData>>;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// File name used for the on-disk cache.
pub const CACHE_FILE_NAME: &str = "israel-public-transportation.zip";

const GTFS_URL: &str = "https://gtfs.mot.gov.il/gtfsfiles/israel-public-transportation.zip";

/// How often the database is refreshed in the background (~30 days).
const REFRESH_INTERVAL: Duration = Duration::from_secs(30 * 24 * 60 * 60);

/// Maximum age of the on-disk cache before a fresh download is triggered.
const CACHE_MAX_AGE_SECS: u64 = 7 * 24 * 60 * 60; // 1 week

// ---------------------------------------------------------------------------
// Public helpers used by main
// ---------------------------------------------------------------------------

/// Returns the platform-specific default cache directory (`<OS cache>/harail`).
pub fn default_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("harail")
}

/// On startup: returns a parsed [`RailroadData`] from the on-disk cache when
/// it is younger than one week, otherwise downloads a fresh copy first.
pub async fn load_or_download(
    cache_path: &Path,
) -> Result<RailroadData, Box<dyn std::error::Error + Send + Sync>> {
    if cache_is_fresh(cache_path) {
        println!(
            "Cache is fresh (< 7 days old) — loading from {}…",
            cache_path.display()
        );
        println!("Parsing GTFS data…");
        match parse_zip_file(cache_path.to_path_buf()).await {
            Ok(data) => {
                println!("GTFS data loaded from cache successfully.");
                return Ok(data);
            }
            Err(e) => eprintln!("Cache load failed ({e}); falling back to fresh download…"),
        }
    }
    download_and_cache(cache_path).await
}

/// Background task that wakes every 30 days, downloads a fresh GTFS zip,
/// saves it to `cache_path`, and atomically swaps the in-memory database.
///
/// On failure the existing database is kept and an error is printed; the
/// task continues running and will retry at the next interval.
pub async fn refresh_task(shared: SharedData, cache_path: PathBuf) {
    loop {
        tokio::time::sleep(REFRESH_INTERVAL).await;
        println!("Starting scheduled monthly GTFS refresh…");
        match download_and_cache(&cache_path).await {
            Ok(new_data) => {
                *shared.write().await = new_data;
                println!("GTFS database refreshed and cached successfully.");
            }
            Err(e) => {
                eprintln!("Failed to refresh GTFS data (keeping existing database): {e}");
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Private implementation
// ---------------------------------------------------------------------------

/// Returns `true` when `path` exists and its modification time is younger
/// than [`CACHE_MAX_AGE_SECS`].
fn cache_is_fresh(path: &Path) -> bool {
    let Ok(meta) = std::fs::metadata(path) else {
        return false;
    };
    let Ok(modified) = meta.modified() else {
        return false;
    };
    let Ok(age) = modified.elapsed() else {
        return false;
    };
    age.as_secs() < CACHE_MAX_AGE_SECS
}

/// Parses a GTFS zip that is already on disk into a [`RailroadData`].
///
/// Parsing is CPU-intensive, so this runs on the blocking thread pool.
/// `from_gtfs_zip` returns `Box<dyn Error>` (not `Send`), so the error is
/// stringified inside the closure and re-boxed as `Send + Sync` outside.
async fn parse_zip_file(
    path: PathBuf,
) -> Result<RailroadData, Box<dyn std::error::Error + Send + Sync>> {
    tokio::task::spawn_blocking(move || {
        RailroadData::from_gtfs_zip(&path).map_err(|e| e.to_string())
    })
    .await?
    .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })
}

/// Downloads the GTFS zip from the Ministry of Transport, saves it to
/// `cache_path`, and returns the parsed database.
///
/// The temp file is created inside the cache directory so that the final
/// rename is always on the same filesystem (avoids cross-device move errors).
async fn download_and_cache(
    cache_path: &Path,
) -> Result<RailroadData, Box<dyn std::error::Error + Send + Sync>> {
    // Ensure the cache directory exists.
    if let Some(dir) = cache_path.parent() {
        std::fs::create_dir_all(dir)?;
    }

    println!("Downloading GTFS data from {GTFS_URL} …");
    let response = reqwest::get(GTFS_URL).await?;
    let content_length = response.content_length();

    let cache_dir = cache_path.parent().unwrap_or_else(|| Path::new("."));
    let mut temp_file = tempfile::Builder::new()
        .prefix("gtfs-")
        .suffix(".zip.tmp")
        .tempfile_in(cache_dir)?;

    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        downloaded += chunk.len() as u64;
        std::io::Write::write_all(&mut temp_file, &chunk)?;
        match content_length {
            Some(total) => print!(
                "\r  {downloaded}/{total} bytes ({:.1}%)",
                100.0 * downloaded as f64 / total as f64
            ),
            None => print!("\r  {downloaded} bytes"),
        }
    }
    println!(); // newline after progress line
    println!("Download complete ({downloaded} bytes).");

    // Atomically replace the old cache file.  `persist` uses rename(2) /
    // MoveFileEx, which is atomic on most systems.  On Windows it fails when
    // the target already exists, so we fall back to a plain copy.
    let cache_path_owned = cache_path.to_path_buf();
    if let Err(e) = temp_file.persist(&cache_path_owned) {
        std::fs::copy(e.file.path(), &cache_path_owned)?;
        // `e.file` (the NamedTempFile) is dropped here, cleaning up the temp.
    }
    println!("Cached to {}.", cache_path.display());

    println!("Parsing GTFS data…");
    let data = parse_zip_file(cache_path_owned).await?;
    println!("GTFS data parsed successfully.");
    Ok(data)
}
