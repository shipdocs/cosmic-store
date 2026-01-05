use crate::app_id::AppId;
use crate::app_info::AppProvide;
use crate::backend::Backends;
use crate::gstreamer::GStreamerCodec;
use crate::search::{SearchResult, SearchSortMode, WaylandFilter};
use crate::{Apps, Message};
use cosmic::action;
use cosmic::app::Task;
use std::sync::Arc;
use std::time::Instant;

pub fn handle_appstream_url(
    apps: &Arc<Apps>,
    backends: &Backends,
    app_stats: &std::collections::HashMap<
        crate::app_id::AppId,
        (u64, Option<crate::app_info::WaylandCompatibility>),
    >,
    os_codename: &str,
    input: String,
    path: &str,
) -> Task<Message> {
    // Handler for appstream:component-id as described in:
    // https://freedesktop.org/software/appstream/docs/sect-AppStream-Misc-URIHandler.html
    let apps = apps.clone();
    let backends = backends.clone();
    let app_stats = app_stats.clone();
    let os_codename = os_codename.to_string();
    let component_id = AppId::new(path.trim_start_matches('/'));
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || {
                let start = Instant::now();
                let results = crate::search_logic::generic_search(
                    &apps,
                    &backends,
                    &app_stats,
                    &os_codename,
                    |id, _info, _installed, _stats_downloads, _stats_compat| {
                        //TODO: fuzzy search with lower weight?
                        if id == &component_id { Some(0) } else { None }
                    },
                    SearchSortMode::Relevance,
                    WaylandFilter::All,
                );
                let duration = start.elapsed();
                log::info!(
                    "searched for ID {:?} in {:?}, found {} results",
                    component_id,
                    duration,
                    results.len()
                );
                action::app(Message::SearchResults(input, results, true))
            })
            .await
            .unwrap_or(action::none())
        },
        |x| x,
    )
}

pub fn handle_file_url(backends: &Backends, input: String, path: &str) -> Task<Message> {
    let path = path.to_string();
    let backends = backends.clone();
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || {
                let start = Instant::now();
                let mut packages = Vec::new();
                for (backend_name, backend) in backends.iter() {
                    match backend.file_packages(&path) {
                        Ok(backend_packages) => {
                            for package in backend_packages {
                                packages.push((backend_name, package));
                            }
                        }
                        Err(err) => {
                            log::warn!(
                                "failed to load file {:?} using backend {:?}: {}",
                                path,
                                backend_name,
                                err
                            );
                        }
                    }
                }
                let duration = start.elapsed();
                log::info!(
                    "loaded file {:?} in {:?}, found {} packages",
                    path,
                    duration,
                    packages.len()
                );

                //TODO: store the resolved packages somewhere
                let mut results = Vec::with_capacity(packages.len());
                for (backend_name, package) in packages {
                    results.push(SearchResult::new(
                        backend_name,
                        package.id,
                        Some(package.icon),
                        package.info,
                        0,
                    ));
                }
                action::app(Message::SearchResults(input, results, true))
            })
            .await
            .unwrap_or(action::none())
        },
        |x| x,
    )
}

pub fn handle_gstreamer_codec(
    backends: &Backends,
    input: String,
    gstreamer_codec: GStreamerCodec,
) -> Task<Message> {
    let backends = backends.clone();
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || {
                let start = Instant::now();
                let mut packages = Vec::new();
                for (backend_name, backend) in backends.iter() {
                    match backend.gstreamer_packages(&gstreamer_codec) {
                        Ok(backend_packages) => {
                            for package in backend_packages {
                                packages.push((backend_name, package));
                            }
                        }
                        Err(err) => {
                            log::warn!(
                                "failed to load gstreamer codec {:?} using backend {:?}: {}",
                                gstreamer_codec,
                                backend_name,
                                err
                            );
                        }
                    }
                }
                let duration = start.elapsed();
                log::info!(
                    "loaded gstreamer codec {:?} in {:?}, found {} packages",
                    gstreamer_codec,
                    duration,
                    packages.len()
                );

                //TODO: store the resolved packages somewhere
                let mut results = Vec::with_capacity(packages.len());
                for (backend_name, package) in packages {
                    results.push(SearchResult::new(
                        backend_name,
                        package.id,
                        Some(package.icon),
                        package.info,
                        0,
                    ));
                }
                action::app(Message::SearchResults(input, results, true))
            })
            .await
            .unwrap_or(action::none())
        },
        |x| x,
    )
}

pub fn handle_mime_url(
    apps: &Arc<Apps>,
    backends: &Backends,
    app_stats: &std::collections::HashMap<
        crate::app_id::AppId,
        (u64, Option<crate::app_info::WaylandCompatibility>),
    >,
    os_codename: &str,
    input: String,
    path: &str,
) -> Task<Message> {
    let apps = apps.clone();
    let backends = backends.clone();
    let app_stats = app_stats.clone();
    let os_codename = os_codename.to_string();
    let mime = path.trim_matches('/').to_string();
    let provide = AppProvide::MediaType(mime.clone());
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || {
                let start = Instant::now();
                let results = crate::search_logic::generic_search(
                    &apps,
                    &backends,
                    &app_stats,
                    &os_codename,
                    |_id, info, _installed, stats_downloads, _stats_compat| {
                        //TODO: monthly downloads as weight?
                        if info.provides.contains(&provide) {
                            let downloads = stats_downloads.unwrap_or(info.monthly_downloads);
                            Some(-(downloads as i64))
                        } else {
                            None
                        }
                    },
                    SearchSortMode::Relevance,
                    WaylandFilter::All,
                );
                let duration = start.elapsed();
                log::info!(
                    "searched for mime {:?} in {:?}, found {} results",
                    mime,
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
