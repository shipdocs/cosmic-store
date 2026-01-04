use std::{collections::HashMap, sync::OnceLock, time::Instant};

use crate::app_info::WaylandCompatibility;
use crate::AppId;

const STATS_URL: &str =
    "https://github.com/shipdocs/cosmic-store/releases/latest/download/flathub-stats.bitcode-v0-7";
const STATS_CACHE_PATH: &str = "cosmic-store/flathub-stats.bitcode-v0-7";
const CACHE_MAX_AGE_SECS: u64 = 30 * 24 * 60 * 60; // 30 days

#[derive(bitcode::Decode)]
struct FlathubStatsV7 {
    downloads: HashMap<AppId, u64>,
    compatibility: HashMap<AppId, WaylandCompatibility>,
}

struct FlathubStats {
    downloads: HashMap<AppId, u64>,
    compatibility: HashMap<AppId, WaylandCompatibility>,
}

static STATS: OnceLock<FlathubStats> = OnceLock::new();

fn get_cache_path() -> Option<std::path::PathBuf> {
    Some(dirs::cache_dir()?.join(STATS_CACHE_PATH))
}

fn try_load_cached() -> Option<Vec<u8>> {
    let cache_path = get_cache_path()?;
    let data = std::fs::read(&cache_path).ok()?;
    log::info!("loaded cached flathub statistics from {:?}", cache_path);
    Some(data)
}

fn is_cache_stale() -> bool {
    let Some(cache_path) = get_cache_path() else { return true };
    let Ok(metadata) = std::fs::metadata(&cache_path) else { return true };
    let Ok(modified) = metadata.modified() else { return true };
    let Ok(elapsed) = modified.elapsed() else { return true };
    elapsed.as_secs() >= CACHE_MAX_AGE_SECS
}

fn download_and_cache() -> Option<Vec<u8>> {
    log::info!("downloading flathub statistics from {}", STATS_URL);
    let response = reqwest::blocking::get(STATS_URL).ok()?;
    if !response.status().is_success() {
        log::warn!("failed to download stats: {}", response.status());
        return None;
    }
    let bytes = response.bytes().ok()?.to_vec();

    if let Some(cache_path) = get_cache_path() {
        if let Some(parent) = cache_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if std::fs::write(&cache_path, &bytes).is_ok() {
            log::info!("cached flathub statistics to {:?}", cache_path);
        }
    }
    Some(bytes)
}

fn decode_v7(data: &[u8]) -> Option<FlathubStats> {
    let v7 = bitcode::decode::<FlathubStatsV7>(data).ok()?;
    Some(FlathubStats {
        downloads: v7.downloads,
        compatibility: v7.compatibility,
    })
}

#[cfg(feature = "flathub-stats-v7")]
fn try_load_bundled() -> Option<Vec<u8>> {
    let bundled_path = std::path::Path::new("res/flathub-stats.bitcode-v0-7");
    let data = std::fs::read(bundled_path).ok()?;
    log::info!("loaded bundled flathub statistics");
    Some(data)
}

fn load_stats() -> &'static FlathubStats {
    STATS.get_or_init(|| {
        let start = Instant::now();

        #[cfg(feature = "flathub-stats-v7")]
        {
    // 1. Try cache first (fastest).
            if let Some(data) = try_load_cached() {
                if let Some(stats) = decode_v7(&data) {
                    log::info!("stats ready in {:?}", start.elapsed());
                    return stats;
                }
            }

            // 2. Fall back to bundled file.
            if let Some(data) = try_load_bundled() {
                if let Some(stats) = decode_v7(&data) {
                    log::info!("stats ready in {:?}", start.elapsed());
                    return stats;
                }
            }

            // 3. Last resort: blocking download (only if no cache AND no bundled).
            if let Some(data) = download_and_cache() {
                if let Some(stats) = decode_v7(&data) {
                    log::info!("stats ready in {:?}", start.elapsed());
                    return stats;
                }
            }
        }

        // Legacy v6 fallback.
        let v6_path = std::path::Path::new("res/flathub-stats.bitcode-v0-6");
        match std::fs::read(v6_path)
            .ok()
            .and_then(|data| bitcode::decode::<HashMap<AppId, u64>>(&data).ok())
        {
            Some(downloads) => {
                log::info!("loaded flathub statistics v0-6 in {:?}", start.elapsed());
                FlathubStats {
                    downloads,
                    compatibility: HashMap::new(),
                }
            }
            None => {
                log::warn!("failed to load flathub statistics");
                FlathubStats {
                    downloads: HashMap::new(),
                    compatibility: HashMap::new(),
                }
            }
        }
    })
}

pub fn try_monthly_downloads(id: &AppId) -> Option<u64> {
    STATS.get()?.downloads.get(id).copied()
}

pub fn try_wayland_compatibility(id: &AppId) -> Option<WaylandCompatibility> {
    STATS.get()?.compatibility.get(id).cloned()
}

/// Load stats in background thread, refresh cache if stale.
pub fn load_stats_async() {
    std::thread::spawn(|| {
        let _ = load_stats();

        // If cache is stale, refresh in background for next launch.
        if is_cache_stale() {
            log::info!("cache is stale, refreshing in background");
            let _ = download_and_cache();
        }
    });
}




