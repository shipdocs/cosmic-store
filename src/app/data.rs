use crate::AppId;
use crate::app_entry::Apps;
use crate::app_info::WaylandCompatibility;
use crate::backend::Backends;
use crate::category::Category;
use crate::gstreamer::GStreamerCodec;
use crate::message::Message;
use crate::pages::ExplorePage;
use crate::search::{SearchSortMode, WaylandFilter};
use crate::url_handlers;
use cosmic::action;
use cosmic::app::Task;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

pub fn categories_task(
    apps: Arc<Apps>,
    backends: Backends,
    app_stats: HashMap<AppId, (u64, Option<WaylandCompatibility>)>,
    os_codename: String,
    categories: &'static [Category],
) -> Task<Message> {
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || {
                let start = Instant::now();
                let results = crate::search_logic::categories_results(
                    &apps,
                    &backends,
                    &app_stats,
                    &os_codename,
                    categories,
                );
                let duration = start.elapsed();
                log::info!(
                    "searched for categories {:?} in {:?}, found {} results",
                    categories,
                    duration,
                    results.len()
                );
                action::app(Message::CategoryResults(categories, results))
            })
            .await
            .unwrap_or(action::none())
        },
        |x| x,
    )
}

#[allow(dead_code)]
pub fn explore_results_task(
    apps: Arc<Apps>,
    backends: Backends,
    app_stats: HashMap<AppId, (u64, Option<WaylandCompatibility>)>,
    os_codename: String,
    explore_page: ExplorePage,
) -> Task<Message> {
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || {
                log::info!("start search for {:?}", explore_page);
                let start = Instant::now();
                let now = chrono::Utc::now().timestamp();
                let results = crate::search_logic::explore_results_data(
                    &apps,
                    &backends,
                    &app_stats,
                    &os_codename,
                    explore_page,
                    now,
                );
                let duration = start.elapsed();
                log::info!(
                    "searched for {:?} in {:?}, found {} results",
                    explore_page,
                    duration,
                    results.len()
                );
                action::app(Message::ExploreResults(explore_page, results))
            })
            .await
            .unwrap_or(action::none())
        },
        |x| x,
    )
}

pub fn explore_results_all_batch_task(
    apps: Arc<Apps>,
    backends: Backends,
    app_stats: HashMap<AppId, (u64, Option<WaylandCompatibility>)>,
    os_codename: String,
) -> Task<Message> {
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || {
                log::info!(
                    "start batch search for all explore pages ({} apps)",
                    apps.len()
                );
                let start = Instant::now();
                let now = chrono::Utc::now().timestamp();
                let results_map = crate::search_logic::explore_results_all(
                    &apps,
                    &backends,
                    &app_stats,
                    &os_codename,
                    now,
                );
                let duration = start.elapsed();
                let total_results: usize = results_map.values().map(|v| v.len()).sum();
                log::info!(
                    "batch search for all explore pages in {:?}, found {} total results",
                    duration,
                    total_results
                );
                action::app(Message::ExploreResultsReady(results_map))
            })
            .await
            .unwrap_or(action::none())
        },
        |x| x,
    )
}

pub fn installed_results_task(
    apps: Arc<Apps>,
    backends: Backends,
    app_stats: HashMap<AppId, (u64, Option<WaylandCompatibility>)>,
    os_codename: String,
) -> Task<Message> {
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || {
                let start = Instant::now();
                let results = crate::search_logic::installed_results_data(
                    &apps,
                    &backends,
                    &app_stats,
                    &os_codename,
                );
                let duration = start.elapsed();
                log::info!(
                    "searched for installed in {:?}, found {} results",
                    duration,
                    results.len()
                );
                action::app(Message::InstalledResults(results))
            })
            .await
            .unwrap_or(action::none())
        },
        |x| x,
    )
}

pub fn search_task(
    apps: Arc<Apps>,
    backends: Backends,
    app_stats: HashMap<AppId, (u64, Option<WaylandCompatibility>)>,
    os_codename: String,
    input: String,
    sort_mode: SearchSortMode,
    wayland_filter: WaylandFilter,
) -> Task<Message> {
    // Handle supported URI schemes before trying plain text search
    if let Ok(url) = reqwest::Url::parse(&input) {
        match url.scheme() {
            "appstream" => {
                return url_handlers::handle_appstream_url(
                    &apps,
                    &backends,
                    &app_stats,
                    &os_codename,
                    input,
                    url.path(),
                );
            }
            "file" => {
                return url_handlers::handle_file_url(&backends, input, url.path());
            }
            "mime" => {
                // This is a workaround to be able to search for mime handlers,
                // mime is not a real URL scheme
                return url_handlers::handle_mime_url(
                    &apps,
                    &backends,
                    &app_stats,
                    &os_codename,
                    input,
                    url.path(),
                );
            }
            scheme => {
                log::warn!("unsupported URL scheme {scheme} in {url}");
            }
        }
    }

    // Also handle standard file paths
    if input.starts_with("/") && Path::new(&input).is_file() {
        return url_handlers::handle_file_url(&backends, input.clone(), &input);
    }

    // Also handle gstreamer codec strings
    if let Some(gstreamer_codec) = GStreamerCodec::parse(&input) {
        return url_handlers::handle_gstreamer_codec(&backends, input.clone(), gstreamer_codec);
    }

    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || {
                let start = Instant::now();
                let results = crate::search_logic::search_results(
                    &apps,
                    &backends,
                    &app_stats,
                    &os_codename,
                    &input,
                    sort_mode,
                    wayland_filter,
                );
                let duration = start.elapsed();
                log::info!(
                    "searched for {:?} in {:?}, found {} results",
                    input,
                    duration,
                    results.len()
                );
                action::app(Message::SearchResults(input, results, false))
            })
            .await
            .unwrap_or(action::none())
        },
        |x| x,
    )
}
