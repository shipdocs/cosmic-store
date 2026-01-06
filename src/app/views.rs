use std::cmp;
use std::collections::{BTreeMap, BTreeSet, HashMap};

use cosmic::iced::{Alignment, Length};
use cosmic::widget::segmented_button::SingleSelectModel;
use cosmic::{Element, cosmic_theme, theme, widget};

use crate::app_id::AppId;
use crate::app_info::WaylandCompatibility;
use crate::backend::Package;
use crate::category::Category;
use crate::constants::MAX_RESULTS;
use crate::fl;
use crate::gstreamer::{GStreamerCodec, GStreamerExitCode, Mode};
use crate::icon_cache::icon_cache_handle;
use crate::message::Message;
use crate::operation::{Operation, OperationKind};
use crate::pages::{ContextPage, DialogPage, ExplorePage, NavPage};
use crate::search::{SearchResult, SearchSortMode, WaylandFilter};
use crate::source::{Source, SourceKind};
use crate::ui::{GridMetrics, package_card_view};

pub fn render_search_results<'a>(
    input: &str,
    results: &'a [SearchResult],
    spacing: cosmic_theme::Spacing,
    grid_width: usize,
    app_stats: &'a HashMap<AppId, (u64, Option<WaylandCompatibility>)>,
) -> Element<'a, Message> {
    let results_len = cmp::min(results.len(), MAX_RESULTS);

    let mut column = widget::column::with_capacity(2)
        .padding([0, spacing.space_s, spacing.space_m, spacing.space_s])
        .spacing(spacing.space_xxs)
        .width(Length::Fill);

    if results.is_empty() {
        column = column.push(widget::text::body(fl!("no-results", search = input)));
    }

    column = column.push(SearchResult::grid_view(
        &results[..results_len],
        spacing,
        grid_width,
        Message::SelectSearchResult,
        app_stats,
    ));

    column.into()
}

pub fn render_category_page<'a>(
    nav_page: NavPage,
    category_results: &'a Option<(&'static [Category], Vec<SearchResult>)>,
    sources: &[Source],
    spacing: cosmic_theme::Spacing,
    grid_width: usize,
    app_stats: &'a HashMap<AppId, (u64, Option<WaylandCompatibility>)>,
) -> Element<'a, Message> {
    let cosmic_theme::Spacing {
        space_l,
        space_m,
        space_s,
        space_xxs,
        ..
    } = spacing;
    let mut column = widget::column::with_capacity(3)
        .padding([0, space_s, space_m, space_s])
        .spacing(space_xxs)
        .width(Length::Fill);
    column = column.push(widget::text::title2(nav_page.title()));
    if matches!(nav_page, NavPage::Applets)
        && !sources.is_empty()
        && sources
            .iter()
            .any(|source| matches!(source.kind, SourceKind::Recommended { enabled: false, .. }))
    {
        column = column.push(
            widget::column::with_children(vec![
                widget::Space::with_height(space_m).into(),
                widget::text(fl!("enable-flathub-cosmic")).into(),
                widget::Space::with_height(space_m).into(),
                widget::button::standard(fl!("manage-repositories"))
                    .on_press(Message::ToggleContextPage(ContextPage::Repositories))
                    .into(),
                widget::Space::with_height(space_l).into(),
            ])
            .align_x(Alignment::Center)
            .width(Length::Fill),
        );
    }
    //TODO: ensure category matches?
    match category_results {
        Some((_, results)) => {
            //TODO: paging or dynamic load
            let results_len = cmp::min(results.len(), MAX_RESULTS);

            if results.is_empty() {
                //TODO: no results message?
            }

            column = column.push(SearchResult::grid_view(
                &results[..results_len],
                spacing,
                grid_width,
                Message::SelectCategoryResult,
                app_stats,
            ));
        }
        None => {
            //TODO: loading message?
        }
    }
    column.into()
}

pub fn render_installed_page<'a>(
    installed_results: &'a Option<Vec<SearchResult>>,
    spacing: cosmic_theme::Spacing,
    grid_width: usize,
    app_stats: &'a HashMap<AppId, (u64, Option<WaylandCompatibility>)>,
) -> Element<'a, Message> {
    let mut column = widget::column::with_capacity(3)
        .padding([0, spacing.space_s, spacing.space_m, spacing.space_s])
        .spacing(spacing.space_xxs)
        .width(Length::Fill);
    column = column.push(widget::text::title2(NavPage::Installed.title()));
    match installed_results {
        Some(installed) => {
            if installed.is_empty() {
                column = column.push(widget::text(fl!("no-installed-applications")));
            }

            let GridMetrics {
                cols,
                item_width,
                column_spacing,
            } = SearchResult::grid_metrics(&spacing, grid_width);
            let mut grid = widget::grid();
            let mut col = 0;
            for (installed_i, result) in installed.iter().enumerate() {
                if col >= cols {
                    grid = grid.insert_row();
                    col = 0;
                }
                let mut buttons = Vec::with_capacity(1);
                if let Some(desktop_id) = result.info.desktop_ids.first() {
                    buttons.push(
                        widget::button::standard(fl!("open"))
                            .on_press(Message::OpenDesktopId(desktop_id.clone()))
                            .into(),
                    );
                } else {
                    buttons.push(widget::Space::with_height(Length::Shrink).into());
                }
                grid = grid.push(
                    widget::mouse_area(package_card_view(
                        &result.info,
                        result.icon_opt.as_ref(),
                        buttons,
                        None,
                        &spacing,
                        item_width,
                        app_stats,
                    ))
                    .on_press(Message::SelectInstalled(installed_i)),
                );
                col += 1;
            }
            column = column.push(
                grid.column_spacing(column_spacing)
                    .row_spacing(column_spacing),
            );
        }
        None => {
            //TODO: loading message?
        }
    }
    column.into()
}

pub fn render_updates_page<'a>(
    updates: &'a Option<Vec<(&'static str, Package)>>,
    waiting_installed: &'a Vec<(&'static str, String, AppId)>,
    waiting_updates: &'a Vec<(&'static str, String, AppId)>,
    pending_operations: &'a std::collections::BTreeMap<u64, (Operation, f32)>,
    spacing: cosmic_theme::Spacing,
    grid_width: usize,
    app_stats: &'a HashMap<AppId, (u64, Option<WaylandCompatibility>)>,
) -> Element<'a, Message> {
    let cosmic_theme::Spacing {
        space_l,
        space_m,
        space_s,
        space_xxs,
        ..
    } = spacing;
    let mut column = widget::column::with_capacity(3)
        .padding([0, space_s, space_m, space_s])
        .spacing(space_xxs)
        .width(Length::Fill);
    match updates {
        Some(updates) => {
            if updates.is_empty() {
                column = column
                    .push(widget::text::title2(NavPage::Updates.title()))
                    .push(
                        widget::column::with_children(vec![
                            widget::icon::from_name("system-software-update-symbolic")
                                .size(128)
                                .into(),
                            widget::Space::with_height(space_l).into(),
                            widget::text::title3(fl!("no-updates")).into(),
                            widget::Space::with_height(space_m).into(),
                            widget::button::standard(fl!("check-for-updates"))
                                .on_press(Message::CheckUpdates)
                                .into(),
                        ])
                        .align_x(Alignment::Center),
                    );
            } else {
                column = column.push(
                    widget::row::with_children(vec![
                        widget::text::title2(NavPage::Updates.title()).into(),
                        widget::horizontal_space().into(),
                        widget::button::standard(fl!("update-all"))
                            .on_press(Message::UpdateAll)
                            .into(),
                    ])
                    .align_y(Alignment::Center),
                );

                let GridMetrics {
                    cols,
                    item_width,
                    column_spacing,
                } = Package::grid_metrics(&spacing, grid_width);
                let mut grid = widget::grid();
                let mut col = 0;
                for (updates_i, (backend_name, package)) in updates.iter().enumerate() {
                    let mut controls = Vec::with_capacity(1);
                    let mut top_controls = Vec::with_capacity(1);
                    let mut waiting_refresh = false;
                    for (other_backend_name, source_id, package_id) in
                        waiting_installed.iter().chain(waiting_updates.iter())
                    {
                        if other_backend_name == backend_name
                            && source_id == &package.info.source_id
                            && package_id == &package.id
                        {
                            waiting_refresh = true;
                            break;
                        }
                    }
                    let mut progress_opt = None;
                    for (_id, (op, progress)) in pending_operations.iter() {
                        if &op.backend_name == backend_name
                            && op
                                .infos
                                .iter()
                                .any(|info| info.source_id == package.info.source_id)
                            && op
                                .package_ids
                                .iter()
                                .any(|package_id| package_id == &package.id)
                        {
                            progress_opt = Some(*progress);
                            break;
                        }
                    }
                    if let Some(progress) = progress_opt {
                        controls.push(
                            widget::progress_bar(0.0..=100.0, progress)
                                .height(Length::Fixed(4.0))
                                .into(),
                        );
                    } else if !waiting_refresh {
                        controls.push(
                            widget::button::standard(fl!("update"))
                                .on_press(Message::Operation(
                                    OperationKind::Update,
                                    backend_name,
                                    package.id.clone(),
                                    package.info.clone(),
                                ))
                                .into(),
                        );
                    }
                    top_controls.push(
                        widget::button::icon(widget::icon::from_name("help-info-symbolic"))
                            .on_press(Message::ToggleContextPage(ContextPage::ReleaseNotes(
                                updates_i,
                                package.info.name.clone(),
                            )))
                            .into(),
                    );
                    if col >= cols {
                        grid = grid.insert_row();
                        col = 0;
                    }
                    grid = grid.push(
                        widget::mouse_area(package.card_view(
                            controls,
                            Some(top_controls),
                            &spacing,
                            item_width,
                            app_stats,
                        ))
                        .on_press(Message::SelectUpdates(updates_i)),
                    );
                    col += 1;
                }
                column = column.push(
                    grid.column_spacing(column_spacing)
                        .row_spacing(column_spacing),
                );
            }
        }
        None => {
            column = column
                .push(widget::text::title2(NavPage::Updates.title()))
                .push(
                    widget::column::with_children(vec![
                        widget::Space::with_height(space_l).into(),
                        widget::text(fl!("checking-for-updates")).into(),
                    ])
                    .align_x(Alignment::Center),
                );
        }
    }
    column.into()
}

pub fn render_explore_page<'a>(
    explore_page_opt: &'a Option<ExplorePage>,
    explore_results: &'a HashMap<ExplorePage, Vec<SearchResult>>,
    loading_frame: usize,
    spacing: cosmic_theme::Spacing,
    grid_width: usize,
    viewport_height: f32,
    app_stats: &'a HashMap<AppId, (u64, Option<WaylandCompatibility>)>,
) -> Element<'a, Message> {
    let cosmic_theme::Spacing {
        space_s,
        space_m,
        space_xxs,
        ..
    } = spacing;

    match explore_page_opt {
        Some(explore_page) => {
            let mut column = widget::column::with_capacity(3)
                .padding([0, space_s, space_m, space_s])
                .spacing(space_xxs)
                .width(Length::Fill);
            column = column.push(
                widget::button::text(NavPage::Explore.title())
                    .leading_icon(icon_cache_handle("go-previous-symbolic", 16))
                    .on_press(Message::ExplorePage(None)),
            );
            column = column.push(widget::text::title4(explore_page.title()));
            //TODO: ensure explore_page matches
            match explore_results.get(explore_page) {
                Some(results) => {
                    //TODO: paging or dynamic load
                    let results_len = cmp::min(results.len(), MAX_RESULTS);

                    if results.is_empty() {
                        //TODO: no results message?
                    }
                    column = column.push(SearchResult::grid_view(
                        &results[..results_len],
                        spacing,
                        grid_width,
                        move |result_i| Message::SelectExploreResult(*explore_page, result_i),
                        app_stats,
                    ));
                }
                None => {
                    column = column.push(
                        widget::container(
                            widget::column::with_children(vec![
                                widget::icon::from_name("com.system76.CosmicStore")
                                    .size(128)
                                    .into(),
                                widget::Space::with_height(spacing.space_l).into(),
                                widget::text::title3(fl!("loading")).into(),
                                widget::Space::with_height(spacing.space_xs).into(),
                                widget::progress_bar(0.0..=100.0, {
                                    let cycle = (loading_frame % 200) as f32;
                                    if cycle < 100.0 { cycle } else { 200.0 - cycle }
                                })
                                .width(Length::Fixed(200.0))
                                .into(),
                            ])
                            .align_x(Alignment::Center),
                        )
                        .width(Length::Fill)
                        .height(Length::Fixed(viewport_height))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center),
                    );
                }
            }
            column.into()
        }
        None => {
            let explore_pages = ExplorePage::all();
            let mut column = widget::column::with_capacity(explore_pages.len() * 2)
                .padding([0, space_s, space_m, space_s])
                .spacing(space_xxs)
                .width(Length::Fill);
            if explore_results.is_empty() {
                column = column.push(
                    widget::container(
                        widget::column::with_children(vec![
                            widget::icon::from_name("com.system76.CosmicStore")
                                .size(128)
                                .into(),
                            widget::Space::with_height(spacing.space_l).into(),
                            widget::text::title3(fl!("loading")).into(),
                            widget::Space::with_height(spacing.space_xs).into(),
                            widget::progress_bar(0.0..=100.0, {
                                let cycle = (loading_frame % 200) as f32;
                                if cycle < 100.0 { cycle } else { 200.0 - cycle }
                            })
                            .width(Length::Fixed(200.0))
                            .into(),
                        ])
                        .align_x(Alignment::Center),
                    )
                    .width(Length::Fill)
                    .height(Length::Fixed(viewport_height))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
                );
            } else {
                for explore_page in explore_pages.iter() {
                    //TODO: ensure explore_page matches
                    match explore_results.get(explore_page) {
                        Some(results) if !results.is_empty() => {
                            let GridMetrics { cols, .. } =
                                SearchResult::grid_metrics(&spacing, grid_width);

                            let max_results = match cols {
                                1 => 4,
                                2 => 8,
                                3 => 9,
                                _ => cols * 2,
                            };

                            //TODO: adjust results length based on app size?
                            let results_len = cmp::min(results.len(), max_results);

                            column = column.push(widget::row::with_children(vec![
                                widget::text::title4(explore_page.title()).into(),
                                widget::horizontal_space().into(),
                                widget::button::text(fl!("see-all"))
                                    .trailing_icon(icon_cache_handle("go-next-symbolic", 16))
                                    .on_press(Message::ExplorePage(Some(*explore_page)))
                                    .into(),
                            ]));

                            column = column.push(SearchResult::grid_view(
                                &results[..results_len],
                                spacing,
                                grid_width,
                                |result_i| Message::SelectExploreResult(*explore_page, result_i),
                                app_stats,
                            ));
                        }
                        _ => {}
                    }
                }
            }
            column.into()
        }
    }
}

pub fn render_dialog<'a>(
    dialog_page: &'a DialogPage,
    failed_operations: &'a BTreeMap<u64, (Operation, f32, String)>,
    size: Option<cosmic::iced::Size>,
    uninstall_purge_data: bool,
    applet_placement_buttons: &'a SingleSelectModel,
    app_id: &str,
) -> Option<Element<'a, Message>> {
    let dialog = match dialog_page {
        DialogPage::FailedOperation(id) => {
            //TODO: try next dialog page (making sure index is used by Dialog messages)?
            let (operation, _, err) = failed_operations.get(id)?;

            let (title, body) = operation.failed_dialog(err);
            widget::dialog()
                .title(title)
                .body(body)
                .icon(widget::icon::from_name("dialog-error").size(64))
                //TODO: retry action
                .primary_action(
                    widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                )
        }
        DialogPage::RepositoryAddError(err) => {
            widget::dialog()
                .title(fl!("repository-add-error-title"))
                .body(err)
                .icon(widget::icon::from_name("dialog-error").size(64))
                //TODO: retry action
                .primary_action(
                    widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                )
        }
        DialogPage::RepositoryRemove(_backend_name, repo_rm) => {
            let mut list = widget::list::list_column();
            //TODO: fix max dialog height in libcosmic?
            let mut scrollable_height = 0.0;
            for (i, (_id, name)) in repo_rm.installed.iter().enumerate() {
                if i > 0 {
                    //TODO: add correct padding per item
                    scrollable_height += 0.0;
                }
                //TODO: show icons
                list = list.add(widget::text(name));
                scrollable_height += 32.0;
            }
            widget::dialog()
                .title(fl!(
                    "repository-remove-title",
                    name = repo_rm.rms[0].name.as_str()
                ))
                .body(fl!(
                    "repository-remove-body",
                    dependency = repo_rm.rms.get(1).map_or("none", |rm| rm.name.as_str())
                ))
                .control(widget::scrollable(list).height(if let Some(size) = size {
                    let max_size = (size.height - 192.0).min(480.0);
                    if scrollable_height > max_size {
                        Length::Fixed(max_size)
                    } else {
                        Length::Shrink
                    }
                } else {
                    Length::Fill
                }))
                .primary_action(
                    widget::button::destructive(fl!("remove")).on_press(Message::DialogConfirm),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                )
        }
        DialogPage::Uninstall(backend_name, _id, info) => {
            let is_flatpak = backend_name.starts_with("flatpak");
            let mut dialog = widget::dialog()
                .title(fl!("uninstall-app", name = info.name.as_str()))
                .body(if is_flatpak {
                    fl!("uninstall-app-flatpak-warning", name = info.name.as_str())
                } else {
                    fl!("uninstall-app-warning", name = info.name.as_str())
                })
                .icon(widget::icon::from_name(app_id).size(64));

            // Only show data deletion option for Flatpak apps
            if is_flatpak {
                dialog = dialog.control(
                    widget::checkbox(fl!("delete-app-data"), uninstall_purge_data)
                        .on_toggle(Message::ToggleUninstallPurgeData),
                );
            }

            dialog
                .primary_action(
                    widget::button::destructive(fl!("uninstall")).on_press(Message::DialogConfirm),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                )
        }
        DialogPage::Place(id) => widget::dialog()
            .title(fl!("place-applet"))
            .body(fl!("place-applet-desc"))
            .control(
                widget::row().push(
                    cosmic::widget::segmented_control::horizontal(applet_placement_buttons)
                        .on_activate(Message::SelectPlacement)
                        .minimum_button_width(0),
                ),
            )
            .primary_action(
                widget::button::suggested(fl!("place-and-refine"))
                    .on_press(Message::PlaceApplet(id.clone())),
            )
            .secondary_action(
                widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
            ),
    };

    Some(dialog.into())
}

pub fn render_footer<'a>(
    progress_operations: &BTreeSet<u64>,
    pending_operations: &'a BTreeMap<u64, (Operation, f32)>,
    complete_operations: &BTreeMap<u64, Operation>,
) -> Option<Element<'a, Message>> {
    if progress_operations.is_empty() {
        return None;
    }

    let cosmic_theme::Spacing {
        space_xxs,
        space_xs,
        space_s,
        ..
    } = theme::active().cosmic().spacing;

    let mut title = String::new();
    let mut total_progress = 0.0;
    let mut count = 0;
    for (_id, (op, progress)) in pending_operations.iter() {
        if title.is_empty() {
            title = op.pending_text(*progress as i32);
        }
        total_progress += progress;
        count += 1;
    }
    let running = count;
    // Adjust the progress bar so it does not jump around when operations finish
    for id in progress_operations.iter() {
        if complete_operations.contains_key(id) {
            total_progress += 100.0;
            count += 1;
        }
    }
    let finished = count - running;
    total_progress /= count as f32;
    if running > 1 {
        if finished > 0 {
            title = fl!(
                "operations-running-finished",
                running = running,
                finished = finished,
                percent = (total_progress as i32)
            );
        } else {
            title = fl!(
                "operations-running",
                running = running,
                percent = (total_progress as i32)
            );
        }
    }

    //TODO: get height from theme?
    let progress_bar_height = Length::Fixed(4.0);
    let progress_bar =
        widget::progress_bar(0.0..=100.0, total_progress).height(progress_bar_height);

    let container = widget::layer_container(widget::column::with_children(vec![
        progress_bar.into(),
        widget::Space::with_height(space_xs).into(),
        widget::text::body(title).into(),
        widget::Space::with_height(space_s).into(),
        widget::row::with_children(vec![
            widget::button::link(fl!("details"))
                .on_press(Message::ToggleContextPage(ContextPage::Operations))
                .padding(0)
                .trailing_icon(true)
                .into(),
            widget::horizontal_space().into(),
            widget::button::standard(fl!("dismiss"))
                .on_press(Message::PendingDismiss)
                .into(),
        ])
        .align_y(Alignment::Center)
        .into(),
    ]))
    .padding([space_xxs, space_xs])
    .layer(cosmic_theme::Layer::Primary);

    Some(container.into())
}

#[allow(clippy::too_many_arguments)]
pub fn render_header_start<'a>(
    mode: &Mode,
    search_active: bool,
    search_input: &'a str,
    search_id: widget::Id,
    search_sort_options: &'a [String],
    search_sort_mode: SearchSortMode,
    wayland_filter_options: &'a [String],
    wayland_filter: WaylandFilter,
) -> Vec<Element<'a, Message>> {
    match mode {
        Mode::Normal => {
            if search_active {
                vec![
                    widget::text_input::search_input("", search_input)
                        .width(Length::Fixed(240.0))
                        .id(search_id)
                        .on_clear(Message::SearchClear)
                        .on_input(Message::SearchInput)
                        .on_submit(Message::SearchSubmit)
                        .into(),
                    widget::dropdown(
                        search_sort_options,
                        Some(match search_sort_mode {
                            SearchSortMode::Relevance => 0,
                            SearchSortMode::MostDownloads => 1,
                            SearchSortMode::RecentlyUpdated => 2,
                            SearchSortMode::BestWaylandSupport => 3,
                        }),
                        |index| match index {
                            0 => Message::SearchSortMode(SearchSortMode::Relevance),
                            1 => Message::SearchSortMode(SearchSortMode::MostDownloads),
                            2 => Message::SearchSortMode(SearchSortMode::RecentlyUpdated),
                            _ => Message::SearchSortMode(SearchSortMode::BestWaylandSupport),
                        },
                    )
                    .width(Length::Fixed(200.0))
                    .into(),
                    widget::dropdown(
                        wayland_filter_options,
                        Some(match wayland_filter {
                            WaylandFilter::All => 0,
                            WaylandFilter::Excellent => 1,
                            WaylandFilter::Good => 2,
                            WaylandFilter::Caution => 3,
                            WaylandFilter::Limited => 4,
                            WaylandFilter::Unknown => 5,
                        }),
                        |index| match index {
                            0 => Message::WaylandFilter(WaylandFilter::All),
                            1 => Message::WaylandFilter(WaylandFilter::Excellent),
                            2 => Message::WaylandFilter(WaylandFilter::Good),
                            3 => Message::WaylandFilter(WaylandFilter::Caution),
                            4 => Message::WaylandFilter(WaylandFilter::Limited),
                            _ => Message::WaylandFilter(WaylandFilter::Unknown),
                        },
                    )
                    .width(Length::Fixed(200.0))
                    .into(),
                ]
            } else {
                vec![
                    widget::button::icon(widget::icon::from_name("system-search-symbolic"))
                        .on_press(Message::SearchActivate)
                        .padding(8)
                        .into(),
                ]
            }
        }
        Mode::GStreamer { .. } => Vec::new(),
    }
}

pub fn render_header_end<'a>(mode: &Mode) -> Vec<Element<'a, Message>> {
    match mode {
        Mode::Normal => {
            vec![
                widget::tooltip(
                    widget::button::icon(widget::icon::from_name("application-menu-symbolic"))
                        .on_press(Message::ToggleContextPage(ContextPage::Repositories)),
                    widget::text(fl!("manage-repositories")),
                    widget::tooltip::Position::Bottom,
                )
                .into(),
            ]
        }
        Mode::GStreamer { .. } => Vec::new(),
    }
}

pub fn render_gstreamer_view<'a>(
    codec: &GStreamerCodec,
    selected: &BTreeSet<usize>,
    installing: bool,
    pending_operations: &'a BTreeMap<u64, (Operation, f32)>,
    failed_operations: &'a BTreeMap<u64, (Operation, f32, String)>,
    complete_operations: &BTreeMap<u64, Operation>,
    search_results: &'a Option<(String, Vec<SearchResult>)>,
) -> Element<'a, Message> {
    let cosmic_theme::Spacing {
        space_s,
        space_xxs,
        space_xs,
        ..
    } = theme::active().cosmic().spacing;

    //TODO: share code with DialogPage?
    let mut dialog = widget::dialog()
        .icon(widget::icon::from_name("dialog-question").size(64))
        .title(fl!("codec-title"))
        .body(fl!(
            "codec-header",
            application = codec.application.as_str(),
            description = codec.description.as_str()
        ));
    if installing {
        let mut list = widget::list_column();

        for (_id, (op, progress)) in pending_operations.iter().rev() {
            list = list.add(widget::column::with_children(vec![
                widget::progress_bar(0.0..=100.0, *progress)
                    .height(Length::Fixed(4.0))
                    .into(),
                widget::Space::with_height(space_xs).into(),
                widget::text(op.pending_text(*progress as i32)).into(),
            ]));
        }

        for (_id, (op, progress, error)) in failed_operations.iter().rev() {
            list = list.add(widget::column::with_children(vec![
                widget::text(op.pending_text(*progress as i32)).into(),
                widget::text(error).into(),
            ]));
        }

        for (_id, op) in complete_operations.iter().rev() {
            list = list.add(widget::text(op.completed_text()));
        }

        dialog = dialog.control(widget::scrollable(list));
        if pending_operations.is_empty() {
            let code = if failed_operations.is_empty() {
                dialog = dialog.control(widget::text(fl!("codec-installed")));
                GStreamerExitCode::Success
            } else {
                dialog = dialog.control(widget::text(fl!("codec-error")));
                GStreamerExitCode::Error
            };
            dialog = dialog.secondary_action(
                widget::button::standard(fl!("close")).on_press(Message::GStreamerExit(code)),
            );
        }
    } else {
        match search_results {
            Some((_input, results)) => {
                let mut list = widget::list_column();
                for (i, result) in results.iter().enumerate() {
                    list = list.add(
                        widget::mouse_area(
                            widget::button::custom(
                                widget::row::with_children(vec![
                                    widget::column::with_children(vec![
                                        widget::text::body(&result.info.name).into(),
                                        widget::text::caption(&result.info.summary).into(),
                                    ])
                                    .into(),
                                    widget::horizontal_space().into(),
                                    if selected.contains(&i) {
                                        widget::icon::from_name("checkbox-checked-symbolic")
                                            .size(16)
                                            .into()
                                    } else {
                                        widget::Space::with_width(Length::Fixed(16.0)).into()
                                    },
                                ])
                                .spacing(space_s)
                                .align_y(Alignment::Center),
                            )
                            .width(Length::Fill)
                            .class(theme::Button::MenuItem)
                            .force_enabled(true),
                        )
                        .on_press(Message::GStreamerToggle(i)),
                    );
                }
                dialog = dialog.control(widget::scrollable(list)).control(
                    widget::row::with_children(vec![
                        widget::icon::from_name("dialog-warning").size(16).into(),
                        widget::text(fl!("codec-footer")).into(),
                    ])
                    .spacing(space_xxs),
                );
            }
            None => {
                //TODO: loading indicator?
                //column = column.push(widget::text("Loading..."));
            }
        }
        let mut install_button = widget::button::suggested(fl!("install"));
        if !selected.is_empty() {
            install_button = install_button.on_press(Message::GStreamerInstall);
        }
        dialog = dialog.primary_action(install_button).secondary_action(
            widget::button::standard(fl!("cancel"))
                .on_press(Message::GStreamerExit(GStreamerExitCode::UserAbort)),
        )
    }
    dialog
        .control(widget::vertical_space())
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
