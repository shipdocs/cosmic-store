//! External storefront discovery. Purchases and installation stay in the storefront.
use crate::app_entry::{AppEntry, Apps};
use crate::app_id::AppId;
use crate::app_info::{AppInfo, AppScreenshot, AppUrl};
use crate::search::SearchResult;
use rayon::prelude::*;
use serde_json::Value;
use std::{collections::HashSet, error::Error, fs, path::PathBuf, sync::Arc, time::Duration};

pub const STEAM: &str = "steam";
pub const NATIVE_LINUX: &str = "X-ShipDocs-NativeLinux";
pub const NEW_RELEASE: &str = "X-ShipDocs-NewRelease";

fn client() -> Result<reqwest::blocking::Client, reqwest::Error> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(8))
        .connect_timeout(Duration::from_secs(3))
        .user_agent("Kompas/0.1")
        .build()
}

fn cache_path() -> Option<PathBuf> {
    Some(dirs::cache_dir()?.join("cosmic-store/steam-featured-v2.json"))
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

fn software_type(kind: &str) -> bool {
    matches!(kind, "game" | "dlc" | "demo" | "software")
}

// Featured/search feeds call hardware an app too. Validate using the product's real type.
// Cache just the needed facts; expired successful metadata is usable offline.
fn validated_item(client: &reqwest::blocking::Client, item: &Value) -> Option<Value> {
    if item
        .get("type")
        .is_some_and(|t| t != "app" && t.as_u64() != Some(0))
    {
        return None;
    }
    let id = item.get("id")?.as_u64()?;
    if id == 0 {
        return None;
    }
    let path = dirs::cache_dir().map(|p| p.join(format!("cosmic-store/steam-metadata/{id}.json")));
    let cached = path
        .as_ref()
        .and_then(|p| fs::read(p).ok())
        .and_then(|b| serde_json::from_slice::<Value>(&b).ok());
    let fresh = path
        .as_ref()
        .and_then(|p| fs::metadata(p).ok())
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.elapsed().ok())
        .is_some_and(|age| age < Duration::from_secs(86400));
    let facts = if fresh {
        cached
    } else {
        let fetched = client.get("https://store.steampowered.com/api/appdetails/")
            .query(&[("appids", id.to_string()), ("cc", "nl".to_string()), ("l", "english".to_string())])
            .send().ok().and_then(|r| r.error_for_status().ok()).and_then(|r| r.json::<Value>().ok())
            .and_then(|v| {
                let data = v.get(id.to_string())?.get("data")?;
                Some(serde_json::json!({"type": data.get("type")?, "platforms": data.get("platforms"), "controller_support": data.get("controller_support")}))
            });
        if let (Some(facts), Some(path)) = (&fetched, &path) {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Ok(bytes) = serde_json::to_vec(facts) {
                let _ = fs::write(path, bytes);
            }
        }
        fetched.or(cached)
    }?;
    let kind = facts.get("type")?.as_str()?;
    if !software_type(kind) {
        return None;
    }
    let mut item = item.clone();
    item["catalog_type"] = Value::String(kind.to_string());
    item["platforms"] = facts["platforms"].clone();
    // Remove the generic featured feed flag in favor of validated platform metadata.
    if let Some(map) = item.as_object_mut() {
        map.remove("linux_available");
    }
    item["controller_support"] = facts["controller_support"].clone();
    Some(item)
}

fn validate_items(items: Vec<Value>) -> Vec<Value> {
    let Ok(client) = client() else {
        return Vec::new();
    };
    let Ok(pool) = rayon::ThreadPoolBuilder::new().num_threads(6).build() else {
        return Vec::new();
    };
    pool.install(|| {
        items
            .par_iter()
            .filter_map(|item| validated_item(&client, item))
            .collect()
    })
}

fn validate_featured(mut value: Value) -> Value {
    let mut items = Vec::new();
    let mut ids = HashSet::new();
    for section in ["top_sellers", "new_releases", "specials"] {
        if let Some(feed) = value
            .pointer(&format!("/{section}/items"))
            .and_then(Value::as_array)
        {
            for item in feed.iter().take(16) {
                if let Some(id) = item.get("id").and_then(Value::as_u64) {
                    if ids.insert(id) {
                        items.push(item.clone());
                    }
                }
            }
        }
    }
    let verified: std::collections::HashMap<_, _> = validate_items(items)
        .into_iter()
        .filter_map(|v| Some((v.get("id")?.as_u64()?, v)))
        .collect();
    for section in ["top_sellers", "new_releases", "specials"] {
        if let Some(feed) = value
            .pointer_mut(&format!("/{section}/items"))
            .and_then(Value::as_array_mut)
        {
            *feed = feed
                .iter()
                .filter_map(|item| verified.get(&item.get("id")?.as_u64()?).cloned())
                .collect();
        }
    }
    value
}

fn item_info(item: &Value, new_release: bool) -> Option<(AppId, Arc<AppInfo>)> {
    // Featured feeds also contain bundles and packages; those IDs are not app IDs.
    if item
        .get("type")
        .is_some_and(|t| t != "app" && t.as_u64() != Some(0))
    {
        return None;
    }
    if item
        .get("catalog_type")
        .and_then(Value::as_str)
        .is_some_and(|kind| !software_type(kind))
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
    let mut categories = vec![
        if item.get("catalog_type").and_then(Value::as_str) == Some("software") {
            "Utility"
        } else {
            "Game"
        }
        .to_string(),
    ];
    if linux {
        categories.push(NATIVE_LINUX.to_string());
    }
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
            for item in items.iter().take(16) {
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
    if std::env::consts::ARCH != "x86_64" {
        return Apps::new();
    }
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
        .and_then(|r| r.json::<Value>().ok())
        .map(validate_featured)
        .filter(|v| !parse_featured(v).is_empty());
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
    if std::env::consts::ARCH != "x86_64" {
        return Ok(Vec::new());
    }
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
    let items = value
        .get("items")
        .and_then(Value::as_array)
        .map(|items| items.iter().take(30).cloned().collect())
        .unwrap_or_default();
    Ok(validate_items(items)
        .iter()
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

#[cfg(test)]
mod product_type_tests {
    use super::*;
    #[test]
    fn hardware_and_video_are_not_installable_software() {
        for kind in ["hardware", "video", "music", "series"] {
            assert!(!software_type(kind));
            assert!(item_info(&serde_json::json!({"id":42,"name":"Product","type":0,"catalog_type":kind,"linux_available":true}), false).is_none());
        }
        assert!(software_type("game"));
        assert!(software_type("software"));
    }
}

/// Desktop launchers cover both system and Flatpak installations without running commands.
pub fn steam_desktop_available() -> bool {
    let mut data_dirs = vec![
        PathBuf::from("/usr/share"),
        PathBuf::from("/usr/local/share"),
    ];
    if let Some(home) = dirs::data_dir() {
        data_dirs.push(home);
    }
    if let Some(home) = dirs::home_dir() {
        data_dirs.push(home.join(".local/share/flatpak/exports/share"));
    }
    data_dirs.push(PathBuf::from("/var/lib/flatpak/exports/share"));
    if let Some(paths) = std::env::var_os("XDG_DATA_DIRS") {
        data_dirs.extend(std::env::split_paths(&paths));
    }
    data_dirs.iter().any(|dir| {
        ["steam.desktop", "com.valvesoftware.Steam.desktop"]
            .iter()
            .any(|name| dir.join("applications").join(name).is_file())
    })
}
