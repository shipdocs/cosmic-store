//! External storefront discovery. Purchases and installation stay in the storefront.
use crate::app_entry::{AppEntry, Apps};
use crate::app_id::AppId;
use crate::app_info::{AppInfo, AppScreenshot, AppUrl};
use crate::search::SearchResult;
use rayon::prelude::*;
use serde_json::Value;
use std::{collections::HashSet, error::Error, fs, path::PathBuf, sync::Arc, time::Duration};

pub const STEAM: &str = "steam";
pub const NEW_RELEASE: &str = "X-ShipDocs-NewRelease";

fn client() -> Result<reqwest::blocking::Client, reqwest::Error> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(8))
        .connect_timeout(Duration::from_secs(3))
        .user_agent("ShipDocsStore/0.1")
        .build()
}

fn cache_path() -> Option<PathBuf> {
    Some(dirs::cache_dir()?.join("cosmic-store/steam-featured.json"))
}

pub fn steam_id(info: &AppInfo) -> Option<u64> {
    if info.source_id != STEAM {
        return None;
    }
    info.desktop_ids
        .first()?
        .strip_prefix("steam.")?
        .parse()
        .ok()
}

pub fn store_url(id: u64) -> String {
    format!("https://store.steampowered.com/app/{id}/")
}
pub fn install_url(id: u64) -> String {
    format!("steam://install/{id}")
}

fn item_info(item: &Value, new_release: bool) -> Option<(AppId, Arc<AppInfo>)> {
    // Featured feeds also contain bundles and packages; those IDs are not app IDs.
    if item
        .get("type")
        .is_some_and(|t| t != "app" && t.as_u64() != Some(0))
    {
        return None;
    }
    let id = item.get("id")?.as_u64()?;
    let name = item.get("name")?.as_str()?.trim();
    if id == 0 || name.is_empty() {
        return None;
    }
    let linux = item
        .get("linux_available")
        .and_then(Value::as_bool)
        .or_else(|| item.pointer("/platforms/linux").and_then(Value::as_bool))
        .unwrap_or(false);
    let compatibility = if linux {
        crate::fl!("steam-native")
    } else {
        crate::fl!("steam-unverified")
    };
    let currency = item
        .get("currency")
        .and_then(Value::as_str)
        .or_else(|| item.pointer("/price/currency").and_then(Value::as_str));
    let price = item
        .get("final_price")
        .and_then(Value::as_u64)
        .or_else(|| item.pointer("/price/final").and_then(Value::as_u64));
    let price_text = match (price, currency) {
        (Some(0), _) => crate::fl!("steam-free"),
        (Some(cents), Some("EUR")) => format!("€{}.{:02}", cents / 100, cents % 100),
        (Some(cents), Some(currency)) => format!("{}.{:02} {currency}", cents / 100, cents % 100),
        _ => crate::fl!("steam-check-price"),
    };
    let mut description = format!(
        "{}\n\n{}",
        crate::fl!("steam-handoff-description"),
        compatibility
    );
    if let Some(controller) = item.get("controller_support").and_then(Value::as_str) {
        if controller == "full" || controller == "partial" {
            description.push_str(&format!(
                "\n{}: {controller}",
                crate::fl!("steam-controller")
            ));
        }
    }
    let image = item
        .get("header_image")
        .or_else(|| item.get("large_capsule_image"))
        .or_else(|| item.get("tiny_image"))
        .and_then(Value::as_str);
    let screenshots = image
        .filter(|url| url.starts_with("https://"))
        .map(|url| {
            vec![AppScreenshot {
                caption: name.to_string(),
                url: url.to_string(),
            }]
        })
        .unwrap_or_default();
    let mut categories = vec!["Game".to_string()];
    if new_release {
        categories.push(NEW_RELEASE.to_string());
    }
    let info = AppInfo {
        source_id: STEAM.to_string(),
        source_name: "Steam".to_string(),
        name: name.to_string(),
        summary: format!("{price_text} · {compatibility}"),
        description,
        desktop_ids: vec![format!("steam.{id}")],
        categories,
        screenshots,
        urls: vec![AppUrl::Homepage(store_url(id))],
        ..AppInfo::default()
    };
    Some((AppId::new(&format!("steam.{id}")), Arc::new(info)))
}

fn parse_featured(value: &Value) -> Apps {
    let mut apps = Apps::new();
    for section in ["top_sellers", "new_releases", "specials"] {
        if let Some(items) = value
            .pointer(&format!("/{section}/items"))
            .and_then(Value::as_array)
        {
            for item in items.iter().take(12) {
                if let Some((id, info)) = item_info(item, section == "new_releases") {
                    let entries = apps.entry(id).or_default();
                    if entries.is_empty() || section == "new_releases" {
                        *entries = vec![AppEntry {
                            backend_name: STEAM,
                            info,
                            installed: false,
                        }];
                    }
                }
            }
        }
    }
    apps
}

pub fn featured() -> Apps {
    let cached = cache_path()
        .and_then(|p| fs::read(p).ok())
        .and_then(|b| serde_json::from_slice::<Value>(&b).ok());
    let value = client()
        .ok()
        .and_then(|c| {
            c.get("https://store.steampowered.com/api/featuredcategories/")
                .query(&[("cc", "nl"), ("l", "english")])
                .send()
                .ok()
        })
        .and_then(|r| r.error_for_status().ok())
        .and_then(|r| r.json::<Value>().ok());
    if let (Some(value), Some(path)) = (&value, cache_path()) {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(bytes) = serde_json::to_vec(value) {
            let _ = fs::write(path, bytes);
        }
    }
    value
        .or(cached)
        .map(|v| parse_featured(&v))
        .unwrap_or_default()
}

pub fn search(term: &str) -> Result<Vec<SearchResult>, Box<dyn Error>> {
    let term = if term.trim().eq_ignore_ascii_case("gta") {
        "Grand Theft Auto"
    } else {
        term
    };
    if term.trim().chars().count() < 2 {
        return Ok(Vec::new());
    }
    let value = client()?
        .get("https://store.steampowered.com/api/storesearch/")
        .query(&[("term", term), ("cc", "nl"), ("l", "english")])
        .send()?
        .error_for_status()?
        .json::<Value>()?;
    let mut ids = HashSet::new();
    Ok(value
        .get("items")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .take(30)
        .filter_map(|item| item_info(item, false))
        .filter(|(id, _)| ids.insert(id.clone()))
        .map(|(id, info)| SearchResult::new(STEAM, id, None, info, 0))
        .collect())
}

pub fn image_path(info: &AppInfo) -> Option<PathBuf> {
    Some(dirs::cache_dir()?.join(format!("cosmic-store/steam-images/{}.jpg", steam_id(info)?)))
}

// Runs outside the UI thread. Never fetch an image from a URL typed by the user.
pub fn cache_images(apps: &Apps) {
    apps.par_iter().for_each(|(_, entries)| {
        let Some(info) = entries.first().map(|e| &e.info) else {
            return;
        };
        let Some(path) = image_path(info) else {
            return;
        };
        if path.is_file() {
            return;
        }
        let Some(image) = info.screenshots.first() else {
            return;
        };
        let Ok(c) = client() else {
            return;
        };
        let Ok(response) = c.get(&image.url).send().and_then(|r| r.error_for_status()) else {
            return;
        };
        if response
            .content_length()
            .is_some_and(|size| size > 5 * 1024 * 1024)
        {
            return;
        }
        let Ok(bytes) = response.bytes() else {
            return;
        };
        if bytes.len() > 5 * 1024 * 1024 {
            return;
        }
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(path, bytes);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn filters_packages_and_deduplicates_featured_games() {
        let value = serde_json::json!({"top_sellers":{"items":[{"id":42,"type":0,"name":"Game"},{"id":7,"type":1,"name":"Bundle"}]},"new_releases":{"items":[{"id":42,"type":0,"name":"Game"}]}});
        let apps = parse_featured(&value);
        assert_eq!(apps.len(), 1);
        let info = &apps.values().next().unwrap()[0].info;
        assert!(info.categories.iter().any(|c| c == NEW_RELEASE));
        assert_eq!(steam_id(info), Some(42));
    }
    #[test]
    fn windows_games_are_not_claimed_to_work_on_linux() {
        let (_, info) = item_info(&serde_json::json!({"id":42,"name":"Game","platforms":{"linux":false},"price":{"currency":"EUR","final":1299}}), false).unwrap();
        assert_eq!(info.wayland_compat, None);
        assert!(info.summary.contains("12.99"));
        assert_eq!(install_url(42), "steam://install/42");
    }
    #[test]
    fn ignores_invalid_identifiers() {
        assert!(item_info(&serde_json::json!({"id":0,"name":"Game"}), false).is_none());
        assert!(item_info(&serde_json::json!({"id":"../escape","name":"Game"}), false).is_none());
    }
}

pub fn alternatives(input: &str) -> &'static [&'static str] {
    match input.trim().to_lowercase().as_str() {
        "photoshop" | "adobe photoshop" => &["GIMP", "Krita"],
        "premiere" | "adobe premiere" => &["Kdenlive", "Shotcut"],
        "microsoft office" | "office" => &["LibreOffice", "ONLYOFFICE"],
        _ => &[],
    }
}
