/* Copyright (C) 2020 Yuval Deutscher

* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Database management: downloading, caching, parsing, and periodically
//! refreshing the GTFS feed from the Israel Ministry of Transport.

use chrono::{Local, NaiveTime, TimeZone};
use futures_util::StreamExt;
use harail::RailroadData;
use std::{
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{sync::RwLock, time::Duration};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Thread-safe handle to the live railroad database.
pub type SharedData = Arc<RwLock<RailroadData>>;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// File name of the postcard-serialized database cache.
pub const CACHE_FILE_NAME: &str = "harail.db";

const GTFS_URL: &str = "https://gtfs.mot.gov.il/gtfsfiles/israel-public-transportation.zip";

/// Maximum age of a postcard-serialized database cache before it is considered stale.
const MAX_CACHE_AGE: Duration = Duration::from_secs(7 * 24 * 60 * 60);

// ---------------------------------------------------------------------------
// Public helpers used by main
// ---------------------------------------------------------------------------

/// Returns the platform-specific default cache directory (`<OS cache>/harail`).
pub fn default_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("harail")
}

/// On startup: loads the postcard-serialized cache if it is less than one week
/// old, otherwise downloads, parses and re-caches a fresh GTFS feed.
pub async fn load_or_download(
    cache_path: &Path,
) -> Result<RailroadData, Box<dyn std::error::Error + Send + Sync>> {
    match load_from_cache(cache_path).await {
        Ok(data) if is_cache_fresh(cache_path) => {
            println!("Cache is still valid (less than one week old) — skipping download.");
            return Ok(data);
        }
        Ok(_) => {
            println!("Cache is stale (older than one week) — downloading fresh data…");
        }
        Err(e) => {
            println!("No usable cache ({e}) — downloading fresh data…");
        }
    }
    download_parse_and_cache(cache_path).await
}

/// Background task that wakes every night at 02:00 local time, downloads a
/// fresh GTFS feed, caches the parsed database, and atomically swaps the
/// in-memory database.
///
/// On failure the existing database is kept and an error is printed; the
/// task continues running and will retry at the next 02:00.
pub async fn refresh_task(shared: SharedData, cache_path: PathBuf) {
    loop {
        let sleep_duration = duration_until_next_2am();
        println!(
            "Next GTFS refresh scheduled in {:.1} hours (at 02:00 local time).",
            sleep_duration.as_secs_f64() / 3600.0
        );
        tokio::time::sleep(sleep_duration).await;
        println!("Starting scheduled nightly GTFS refresh…");
        match download_parse_and_cache(&cache_path).await {
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

/// Returns `true` when the cache file exists and was written less than
/// [`MAX_CACHE_AGE`] ago.
fn is_cache_fresh(cache_path: &Path) -> bool {
    std::fs::metadata(cache_path)
        .and_then(|m| m.modified())
        .map(|modified| {
            // elapsed() only fails when mtime is in the future; treat that as fresh.
            modified.elapsed().unwrap_or(Duration::ZERO) < MAX_CACHE_AGE
        })
        .unwrap_or(false)
}

/// Returns the [`Duration`] from now until 02:00 local time tonight, or until
/// 02:00 tomorrow if that time has already passed today.
fn duration_until_next_2am() -> Duration {
    let now = Local::now();
    let target = NaiveTime::from_hms_opt(2, 0, 0).unwrap();

    let today_2am = Local
        .from_local_datetime(&now.date_naive().and_time(target))
        .earliest()
        .unwrap();

    let next_2am = if now < today_2am {
        today_2am
    } else {
        // 02:00 has already passed today — schedule for tomorrow.
        Local
            .from_local_datetime(&(now.date_naive() + chrono::Duration::days(1)).and_time(target))
            .earliest()
            .unwrap()
    };

    let secs = (next_2am - now).num_seconds().max(0) as u64;
    Duration::from_secs(secs)
}

/// Loads and deserializes the postcard-serialized database from disk.
/// Runs on the blocking thread pool so the async executor is not stalled.
async fn load_from_cache(
    cache_path: &Path,
) -> Result<RailroadData, Box<dyn std::error::Error + Send + Sync>> {
    let path = cache_path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
        postcard::from_bytes::<RailroadData>(&bytes).map_err(|e| e.to_string())
    })
    .await?
    .map_err(|e: String| -> Box<dyn std::error::Error + Send + Sync> { e.into() })
}

/// Serializes `data` to `cache_path` using a temp file in the same directory
/// so the final rename is atomic (same filesystem).
fn save_to_cache(data: &RailroadData, cache_path: &Path) -> std::io::Result<()> {
    if let Some(dir) = cache_path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let cache_dir = cache_path.parent().unwrap_or_else(|| Path::new("."));
    let mut temp_file = tempfile::Builder::new()
        .prefix("harail-db-")
        .suffix(".tmp")
        .tempfile_in(cache_dir)?;
    postcard::to_io(data, &mut temp_file).map_err(std::io::Error::other)?;
    temp_file.flush()?;
    // Atomically replace the cache file.  Falls back to copy on Windows when
    // the target already exists (rename-over-existing is not atomic there).
    if let Err(e) = temp_file.persist(cache_path) {
        std::fs::copy(e.file.path(), cache_path)?;
    }
    Ok(())
}

/// Downloads the GTFS zip to a temporary file, then, on the blocking thread
/// pool, parses it with [`RailroadData::from_gtfs_zip`] and saves the result
/// as a postcard-serialized cache.  The zip is discarded after parsing.
async fn download_parse_and_cache(
    cache_path: &Path,
) -> Result<RailroadData, Box<dyn std::error::Error + Send + Sync>> {
    println!("Downloading GTFS data from {GTFS_URL} …");
    let response = reqwest::get(GTFS_URL).await?;
    let content_length = response.content_length();

    // Stream the zip into a temporary file (no need to keep it after parsing).
    let mut temp_zip = tempfile::NamedTempFile::new()?;
    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        downloaded += chunk.len() as u64;
        temp_zip.write_all(&chunk)?;
        match content_length {
            Some(total) => print!(
                "\r  {downloaded}/{total} bytes ({:.1}%)",
                100.0 * downloaded as f64 / total as f64
            ),
            None => print!("\r  {downloaded} bytes"),
        }
    }
    println!();
    println!("Download complete ({downloaded} bytes).");

    // Parse the zip and save the postcard cache on the blocking thread pool.
    // Both operations are CPU/IO-heavy and belong off the async executor.
    // `from_gtfs_zip` returns `Box<dyn Error>` (not Send), so errors are
    // stringified inside the closure and re-boxed as Send + Sync outside.
    println!("Parsing GTFS data…");
    let zip_path = temp_zip.path().to_path_buf();
    let cache_path_owned = cache_path.to_path_buf();
    let data = tokio::task::spawn_blocking(move || {
        let _keep = temp_zip; // keep the temp file alive until parsing is done
        let data = RailroadData::from_gtfs_zip(&zip_path).map_err(|e| e.to_string())?;
        if let Err(e) = save_to_cache(&data, &cache_path_owned) {
            eprintln!("Warning: failed to write database cache: {e}");
        } else {
            println!(
                "Serialized database cached to {}",
                cache_path_owned.display()
            );
        }
        Ok::<_, String>(data)
    })
    .await?
    .map_err(|e: String| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;

    println!("GTFS data parsed successfully.");
    Ok(data)
}
