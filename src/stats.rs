use std::{collections::HashMap, sync::OnceLock, time::Instant};

use crate::app_info::WaylandCompatibility;
use crate::AppId;

const STATS_URL: &str =
    "https://github.com/shipdocs/cosmic-store/releases/latest/download/flathub-stats.bitcode-v0-7";
const STATS_CACHE_PATH: &str = "cosmic-store/flathub-stats.bitcode-v0-7";

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

fn try_download_v7() -> Option<Vec<u8>> {
    let cache_dir = dirs::cache_dir()?;
    let cache_path = cache_dir.join(STATS_CACHE_PATH);

    if let Ok(metadata) = std::fs::metadata(&cache_path) {
        if let Ok(modified) = metadata.modified() {
            if let Ok(elapsed) = modified.elapsed() {
                if elapsed.as_secs() < 30 * 24 * 60 * 60 {
                    log::info!("using cached flathub statistics from {:?}", cache_path);
                    return std::fs::read(&cache_path).ok();
                }
            }
        }
    }

    log::info!("downloading flathub statistics from {}", STATS_URL);
    let response = reqwest::blocking::get(STATS_URL).ok()?;

    if !response.status().is_success() {
        return None;
    }

    let bytes = response.bytes().ok()?.to_vec();
    std::fs::create_dir_all(cache_path.parent()?).ok()?;
    std::fs::write(&cache_path, &bytes).ok()?;

    Some(bytes)
}

fn load_stats() -> &'static FlathubStats {
    STATS.get_or_init(|| {
        let start = Instant::now();

        #[cfg(feature = "flathub-stats-v7")]
        {
            if let Some(data) = try_download_v7() {
                if let Ok(v7) = bitcode::decode::<FlathubStatsV7>(&data) {
                    let elapsed = start.elapsed();
                    log::info!("loaded flathub statistics v0-7 in {:?}", elapsed);
                    return FlathubStats {
                        downloads: v7.downloads,
                        compatibility: v7.compatibility,
                    };
                }
            }

            let bundled_path = std::path::Path::new("res/flathub-stats.bitcode-v0-7");
            if let Ok(data) = std::fs::read(bundled_path) {
                if let Ok(v7) = bitcode::decode::<FlathubStatsV7>(&data) {
                    let elapsed = start.elapsed();
                    log::info!("loaded bundled flathub statistics v0-7 in {:?}", elapsed);
                    return FlathubStats {
                        downloads: v7.downloads,
                        compatibility: v7.compatibility,
                    };
                }
            }
        }

        let v6_path = std::path::Path::new("res/flathub-stats.bitcode-v0-6");
        match std::fs::read(v6_path)
            .ok()
            .and_then(|data| bitcode::decode::<HashMap<AppId, u64>>(&data).ok())
        {
            Some(downloads) => {
                let elapsed = start.elapsed();
                log::info!("loaded flathub statistics v0-6 in {:?}", elapsed);
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

pub fn monthly_downloads(id: &AppId) -> Option<u64> {
    load_stats().downloads.get(id).copied()
}

pub fn wayland_compatibility(id: &AppId) -> Option<WaylandCompatibility> {
    load_stats().compatibility.get(id).cloned()
}
