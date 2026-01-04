use std::{collections::HashMap, error::Error, fs};

use app_id::AppId;
use chrono::{Datelike, Duration, Utc};
#[path = "../../src/app_id.rs"]
mod app_id;

#[derive(serde::Deserialize)]
pub struct Stats {
    refs: HashMap<String, HashMap<String, (u64, u64)>>,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    bitcode::Encode,
    bitcode::Decode,
)]
pub enum WaylandSupport {
    Native,
    Fallback,
    X11Only,
    Unknown,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    bitcode::Encode,
    bitcode::Decode,
)]
pub enum AppFramework {
    Native,
    GTK3,
    GTK4,
    Qt5,
    Qt6,
    QtWebEngine,
    Electron,
    Unknown,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    bitcode::Encode,
    bitcode::Decode,
)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, bitcode::Encode, bitcode::Decode)]
pub struct WaylandCompatibility {
    pub support: WaylandSupport,
    pub framework: AppFramework,
    pub risk_level: RiskLevel,
}

#[derive(serde::Serialize, serde::Deserialize, bitcode::Encode, bitcode::Decode)]
struct FlathubStats {
    generated_at: u64,
    downloads: HashMap<AppId, u64>,
    compatibility: HashMap<AppId, WaylandCompatibility>,
}

#[derive(serde::Serialize)]
struct StatsMetadata {
    version: &'static str,
    generated_at: u64,
    file_size: u64,
    app_count: usize,
}

async fn stats(year: u16, month: u8, day: u8) -> Result<Stats, Box<dyn Error>> {
    let url = format!("https://flathub.org/stats/{year}/{month:02}/{day:02}.json");
    println!("Downloading stats from {}", url);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let body = client.get(&url).send().await?.text().await?;
    let stats = serde_json::from_str::<Stats>(&body)?;
    Ok(stats)
}

fn leap_year(year: u16) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}

async fn fetch_manifest(app_id: &str) -> Result<serde_json::Value, Box<dyn Error>> {
    let branches = ["master", "main", "stable"];

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    for branch in branches {
        let url = format!(
            "https://raw.githubusercontent.com/flathub/{}/{}/{}.json",
            app_id, branch, app_id
        );

        match client.get(&url).send().await {
            Ok(response) if response.status().is_success() => {
                let manifest = response.json().await?;
                return Ok(manifest);
            }
            _ => continue,
        }
    }

    Err(format!("Failed to fetch manifest for {}", app_id).into())
}

fn parse_compatibility(manifest: &serde_json::Value) -> WaylandCompatibility {
    let mut wayland = false;
    let mut x11 = false;
    let mut fallback_x11 = false;

    if let Some(finish_args) = manifest["finish-args"].as_array() {
        for arg in finish_args {
            if let Some(arg_str) = arg.as_str() {
                if arg_str.contains("--socket=wayland") {
                    wayland = true;
                } else if arg_str.contains("--socket=fallback-x11") {
                    fallback_x11 = true;
                } else if arg_str.contains("--socket=x11") {
                    x11 = true;
                }
            }
        }
    }

    let framework = detect_framework(manifest);

    let support = if wayland && !x11 && !fallback_x11 {
        WaylandSupport::Native
    } else if wayland && (fallback_x11 || x11) {
        WaylandSupport::Fallback
    } else if x11 && !wayland {
        WaylandSupport::X11Only
    } else {
        WaylandSupport::Unknown
    };

    let risk_level = calculate_risk_level(support, framework);

    WaylandCompatibility {
        support,
        framework,
        risk_level,
    }
}

fn detect_framework(manifest: &serde_json::Value) -> AppFramework {
    let manifest_str = manifest.to_string().to_lowercase();

    if manifest_str.contains("qtwebengine") || manifest_str.contains("qt5-qtwebengine") {
        return AppFramework::QtWebEngine;
    }
    if manifest_str.contains("electron") {
        return AppFramework::Electron;
    }

    if manifest_str.contains("qt6") || manifest_str.contains("kde6") {
        return AppFramework::Qt6;
    }
    if manifest_str.contains("qt5") || manifest_str.contains("kde5") {
        return AppFramework::Qt5;
    }

    // GTK4 is used by GNOME 40+ (check for gtk-4 or gnome-4x pattern)
    if manifest_str.contains("gtk-4") || manifest_str.contains("gnome-4") {
        return AppFramework::GTK4;
    }
    if manifest_str.contains("gtk-3") || manifest_str.contains("gnome-3") {
        return AppFramework::GTK3;
    }

    AppFramework::Native
}

fn calculate_risk_level(support: WaylandSupport, framework: AppFramework) -> RiskLevel {
    use RiskLevel::*;

    match (support, framework) {
        (WaylandSupport::X11Only, _) => Critical,
        (_, AppFramework::QtWebEngine) => High,
        (_, AppFramework::Electron) => High,
        (WaylandSupport::Native, AppFramework::Qt6) => Medium,
        (WaylandSupport::Fallback, _) => Medium,
        (WaylandSupport::Native, AppFramework::Qt5) => Medium,
        (WaylandSupport::Native, AppFramework::GTK3) => Low,
        (WaylandSupport::Native, AppFramework::GTK4) => Low,
        (WaylandSupport::Native, AppFramework::Native) => Low,
        _ => Medium,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Use previous month's stats (current month data is incomplete)
    let last_month = Utc::now() - Duration::days(30);
    let year = last_month.year() as u16;
    let month = last_month.month() as u8;

    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => panic!("invalid month {}", month),
    };

    println!("Fetching download stats for {}/{}...", year, month);
    let mut ref_downloads = HashMap::<AppId, u64>::new();
    let mut days_fetched = 0;
    for day in 1..=days {
        match stats(year, month, day).await {
            Ok(stats) => {
                days_fetched += 1;
                for (r, archs) in stats.refs {
                    for (_arch, (downloads, _updates)) in archs {
                        let id = r.split('/').next().unwrap();
                        *ref_downloads.entry(AppId::new(id)).or_insert(0) += downloads;
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "Warning: Failed to fetch stats for {}/{}/{}: {}",
                    year, month, day, e
                );
            }
        }
    }
    println!(
        "Fetched stats for {} unique apps ({}/{} days)",
        ref_downloads.len(),
        days_fetched,
        days
    );

    println!(
        "Fetching compatibility data for {} apps...",
        ref_downloads.len()
    );
    let mut compatibility_data = HashMap::<AppId, WaylandCompatibility>::new();

    let mut successful = 0;
    let mut failed = 0;

    for (app_id, _downloads) in ref_downloads.iter() {
        let app_id_str = app_id.raw();

        match fetch_manifest(app_id_str).await {
            Ok(manifest) => {
                let compat = parse_compatibility(&manifest);
                compatibility_data.insert(app_id.clone(), compat);
                successful += 1;

                if successful % 100 == 0 {
                    println!("Processed {}/{} apps...", successful, ref_downloads.len());
                }
            }
            Err(_) => {
                failed += 1;
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    println!(
        "Successfully fetched {} manifests, {} failed",
        successful, failed
    );

    let generated_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let stats = FlathubStats {
        generated_at,
        downloads: ref_downloads.clone(),
        compatibility: compatibility_data,
    };

    let bitcode = bitcode::encode(&stats);

    // Ensure res directory exists
    fs::create_dir_all("../res")?;

    // Write v0-8 bitcode file
    fs::write("../res/flathub-stats.bitcode-v0-8", &bitcode)?;
    println!(
        "Saved to ../res/flathub-stats.bitcode-v0-8 ({} bytes)",
        bitcode.len()
    );

    // Write metadata.json
    let metadata = StatsMetadata {
        version: "v0-8",
        generated_at,
        file_size: bitcode.len() as u64,
        app_count: ref_downloads.len(),
    };
    let metadata_json = serde_json::to_string_pretty(&metadata)?;
    fs::write("../res/flathub-metadata.json", metadata_json)?;
    println!("Saved metadata to ../res/flathub-metadata.json");

    Ok(())
}
