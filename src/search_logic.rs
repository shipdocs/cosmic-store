use crate::app_entry::{AppEntry, Apps};
use crate::app_info::{AppKind, AppProvide, RiskLevel};
use crate::backend::Backends;
use crate::category::Category;
use crate::editors_choice::EDITORS_CHOICE;
use crate::pages::ExplorePage;
// Re-export and use Search types
use crate::localize::LANGUAGE_SORTER;
pub use crate::search::{SearchResult, SearchSortMode, WaylandFilter};
use rayon::prelude::*;
use std::cmp;
use std::path::Path;

/// Pure function moved from App::generic_search
pub fn generic_search<
    F: Fn(&crate::app_id::AppId, &crate::app_info::AppInfo, bool) -> Option<i64> + Send + Sync,
>(
    apps: &Apps,
    backends: &Backends,
    os_codename: &str,
    filter_map: F,
    sort_mode: SearchSortMode,
    wayland_filter: WaylandFilter,
) -> Vec<SearchResult> {
    // We need to access crate::constants::MAX_RESULTS.
    // Since this is a new module, we assuming crate::constants is accessible.
    let max_results = crate::constants::MAX_RESULTS;

    let mut results: Vec<SearchResult> = apps
        .par_iter()
        .filter_map(|(id, infos)| {
            let mut best_weight: Option<i64> = None;
            for AppEntry {
                backend_name,
                info,
                installed,
            } in infos.iter()
            {
                let is_flatpak = backend_name.starts_with("flatpak-");

                if !is_flatpak {
                    if let Some(origin) = &info.origin_opt {
                        if !origin.is_empty() && !origin.contains(os_codename) {
                            /*
                            log::debug!(
                                "Filtering out {} due to origin mismatch: {} (expected {})",
                                info.name,
                                origin,
                                os_codename
                            );
                            */
                            continue;
                        }
                    }
                }

                if let Some(weight) = filter_map(id, info, *installed) {
                    if let Some(prev_weight) = best_weight {
                        if prev_weight <= weight {
                            continue;
                        }
                    }

                    best_weight = Some(weight);
                }
            }
            let weight = best_weight?;
            // Use first info as it is preferred, even if other ones had a higher weight
            let AppEntry {
                backend_name,
                info,
                installed: _,
            } = infos.first()?;

            if wayland_filter != WaylandFilter::All {
                let compat_opt = info.wayland_compat_lazy();
                let matches_filter = match wayland_filter {
                    WaylandFilter::All => true,
                    WaylandFilter::Excellent => compat_opt
                        .map(|c| c.risk_level == RiskLevel::Low)
                        .unwrap_or(false),
                    WaylandFilter::Good => compat_opt
                        .map(|c| c.risk_level == RiskLevel::Medium)
                        .unwrap_or(false),
                    WaylandFilter::Caution => compat_opt
                        .map(|c| c.risk_level == RiskLevel::High)
                        .unwrap_or(false),
                    WaylandFilter::Limited => compat_opt
                        .map(|c| c.risk_level == RiskLevel::Critical)
                        .unwrap_or(false),
                    WaylandFilter::Unknown => compat_opt.is_none(),
                };

                if !matches_filter {
                    return None;
                }
            }

            Some(SearchResult::new(
                backend_name,
                id.clone(),
                None,
                info.clone(),
                weight,
            ))
        })
        .collect();

    match sort_mode {
        SearchSortMode::Relevance => {
            results.par_sort_unstable_by(|a, b| match a.weight.cmp(&b.weight) {
                cmp::Ordering::Equal => match LANGUAGE_SORTER.compare(&a.info.name, &b.info.name) {
                    cmp::Ordering::Equal => {
                        LANGUAGE_SORTER.compare(a.backend_name(), b.backend_name())
                    }
                    ordering => ordering,
                },
                ordering => ordering,
            });
        }
        SearchSortMode::MostDownloads => {
            results.par_sort_unstable_by(|a, b| {
                match b.info.monthly_downloads.cmp(&a.info.monthly_downloads) {
                    cmp::Ordering::Equal => LANGUAGE_SORTER.compare(&a.info.name, &b.info.name),
                    ordering => ordering,
                }
            });
        }
        SearchSortMode::RecentlyUpdated => {
            results.par_sort_unstable_by(|a, b| {
                let a_timestamp = a.info.releases.first().and_then(|r| r.timestamp);
                let b_timestamp = b.info.releases.first().and_then(|r| r.timestamp);
                match b_timestamp.cmp(&a_timestamp) {
                    cmp::Ordering::Equal => LANGUAGE_SORTER.compare(&a.info.name, &b.info.name),
                    ordering => ordering,
                }
            });
        }
        SearchSortMode::BestWaylandSupport => {
            results.par_sort_unstable_by(|a, b| {
                let a_risk = a
                    .info
                    .wayland_compat_lazy()
                    .map(|c| c.risk_level)
                    .unwrap_or(RiskLevel::Critical);
                let b_risk = b
                    .info
                    .wayland_compat_lazy()
                    .map(|c| c.risk_level)
                    .unwrap_or(RiskLevel::Critical);

                // Lower risk level = better (Low=0, Medium=1, High=2, Critical=3)
                let a_score = match a_risk {
                    RiskLevel::Low => 0,
                    RiskLevel::Medium => 1,
                    RiskLevel::High => 2,
                    RiskLevel::Critical => 3,
                };
                let b_score = match b_risk {
                    RiskLevel::Low => 0,
                    RiskLevel::Medium => 1,
                    RiskLevel::High => 2,
                    RiskLevel::Critical => 3,
                };

                match a_score.cmp(&b_score) {
                    cmp::Ordering::Equal => LANGUAGE_SORTER.compare(&a.info.name, &b.info.name),
                    ordering => ordering,
                }
            });
        }
    }
    // Load only enough icons to show one page of results
    //TODO: load in background
    for result in results.iter_mut().take(max_results) {
        let Some(backend) = backends.get(result.backend_name()) else {
            continue;
        };
        let appstream_caches = backend.info_caches();
        let Some(appstream_cache) = appstream_caches
            .iter()
            .find(|x| x.source_id == result.info.source_id)
        else {
            continue;
        };
        result.icon_opt = Some(appstream_cache.icon(&result.info));
    }
    results
}

/// Extracted search logic
pub fn search_results(
    apps: &Apps,
    backends: &Backends,
    os_codename: &str,
    input: &str,
    sort_mode: SearchSortMode,
    wayland_filter: WaylandFilter,
) -> Vec<SearchResult> {
    if input.starts_with("/") && Path::new(&input).is_file() {
        return Vec::new(); // File paths handled by url_handlers in main
    }
    // GStreamer codec handled by url_handlers in main

    let pattern = regex::escape(input);
    let regex = match regex::RegexBuilder::new(&pattern)
        .case_insensitive(true)
        .build()
    {
        Ok(ok) => ok,
        Err(err) => {
            log::warn!("failed to parse regex {:?}: {}", pattern, err);
            return Vec::new();
        }
    };

    generic_search(
        apps,
        backends,
        os_codename,
        |_id, info, _installed| {
            if !matches!(info.kind, AppKind::DesktopApplication) {
                return None;
            }
            //TODO: improve performance
            let stats_weight = |weight: i64| -> i64 {
                //TODO: make sure no overflows
                (weight << 56) - (info.monthly_downloads as i64)
            };

            //TODO: fuzzy match (nucleus-matcher?)
            let regex_weight = |string: &str, weight: i64| -> Option<i64> {
                let mat = regex.find(string)?;
                if mat.range().start == 0 {
                    if mat.range().end == string.len() {
                        Some(stats_weight(weight))
                    } else {
                        Some(stats_weight(weight + 1))
                    }
                } else {
                    Some(stats_weight(weight + 2))
                }
            };
            if let Some(weight) = regex_weight(&info.name, 0) {
                return Some(weight);
            }
            if let Some(weight) = regex_weight(&info.summary, 3) {
                return Some(weight);
            }
            if let Some(weight) = regex_weight(&info.description, 6) {
                return Some(weight);
            }
            None
        },
        sort_mode,
        wayland_filter,
    )
}

/// Extracted categories logic
pub fn categories_results(
    apps: &Apps,
    backends: &Backends,
    os_codename: &str,
    categories: &[Category],
) -> Vec<SearchResult> {
    let applet_provide = AppProvide::Id("com.system76.CosmicApplet".to_string());
    generic_search(
        apps,
        backends,
        os_codename,
        |_id, info, _installed| {
            if !matches!(info.kind, AppKind::DesktopApplication) {
                return None;
            }
            for category in categories {
                //TODO: this hack makes it easier to add applets to the nav bar
                if matches!(category, Category::CosmicApplet) {
                    if info.provides.contains(&applet_provide) {
                        return Some(-(info.monthly_downloads as i64));
                    }
                } else {
                    //TODO: contains doesn't work due to type mismatch
                    if info.categories.iter().any(|x| x == category.id()) {
                        return Some(-(info.monthly_downloads as i64));
                    }
                }
            }
            None
        },
        SearchSortMode::Relevance,
        WaylandFilter::All,
    )
}

/// Extracted explore page logic
pub fn explore_results_data(
    apps: &Apps,
    backends: &Backends,
    os_codename: &str,
    explore_page: ExplorePage,
    now: i64,
) -> Vec<SearchResult> {
    match explore_page {
        ExplorePage::EditorsChoice => generic_search(
            apps,
            backends,
            os_codename,
            |id, _info, _installed| {
                EDITORS_CHOICE
                    .iter()
                    .position(|choice_id| choice_id == &id.normalized())
                    .map(|x| x as i64)
            },
            SearchSortMode::Relevance,
            WaylandFilter::All,
        ),
        ExplorePage::PopularApps => generic_search(
            apps,
            backends,
            os_codename,
            |_id, info, _installed| {
                if !matches!(info.kind, AppKind::DesktopApplication) {
                    return None;
                }
                Some(-(info.monthly_downloads as i64))
            },
            SearchSortMode::Relevance,
            WaylandFilter::All,
        ),
        ExplorePage::MadeForCosmic => {
            let provide = AppProvide::Id("com.system76.CosmicApplication".to_string());
            generic_search(
                apps,
                backends,
                os_codename,
                |_id, info, _installed| {
                    if !matches!(info.kind, AppKind::DesktopApplication) {
                        return None;
                    }
                    if info.provides.contains(&provide) {
                        Some(-(info.monthly_downloads as i64))
                    } else {
                        None
                    }
                },
                SearchSortMode::Relevance,
                WaylandFilter::All,
            )
        }
        ExplorePage::NewApps => generic_search(
            apps,
            backends,
            os_codename,
            |_id, _info, _installed| {
                //TODO
                None
            },
            SearchSortMode::Relevance,
            WaylandFilter::All,
        ),
        ExplorePage::RecentlyUpdated => generic_search(
            apps,
            backends,
            os_codename,
            |id, info, _installed| {
                if !matches!(info.kind, AppKind::DesktopApplication) {
                    return None;
                }
                // Finds the newest release and sorts from newest to oldest
                //TODO: appstream release info is often incomplete
                let mut min_weight = 0;
                for release in info.releases.iter() {
                    if let Some(timestamp) = release.timestamp {
                        if timestamp < now {
                            let weight = -timestamp;
                            if weight < min_weight {
                                min_weight = weight;
                            }
                        } else {
                            log::info!(
                                "{:?} has release timestamp {} which is past the present {}",
                                id,
                                timestamp,
                                now
                            );
                        }
                    }
                }
                Some(min_weight)
            },
            SearchSortMode::Relevance,
            WaylandFilter::All,
        ),
        _ => {
            let categories = explore_page.categories();
            generic_search(
                apps,
                backends,
                os_codename,
                |_id, info, _installed| {
                    if !matches!(info.kind, AppKind::DesktopApplication) {
                        return None;
                    }
                    for category in categories {
                        //TODO: contains doesn't work due to type mismatch
                        if info.categories.iter().any(|x| x == category.id()) {
                            return Some(-(info.monthly_downloads as i64));
                        }
                    }
                    None
                },
                SearchSortMode::Relevance,
                WaylandFilter::All,
            )
        }
    }
}

/// Extracted installed apps logic
pub fn installed_results_data(
    apps: &Apps,
    backends: &Backends,
    os_codename: &str,
) -> Vec<SearchResult> {
    generic_search(
        apps,
        backends,
        os_codename,
        |id, _info, installed| {
            if installed {
                Some(if id.is_system() { -1 } else { 0 })
            } else {
                None
            }
        },
        SearchSortMode::Relevance,
        WaylandFilter::All,
    )
}
