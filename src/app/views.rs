use std::cmp;
use std::collections::HashMap;

use cosmic::iced::{Alignment, Length};
use cosmic::{Element, cosmic_theme, widget};

use crate::app_id::AppId;
use crate::app_info::WaylandCompatibility;
use crate::backend::Package;
use crate::category::Category;
use crate::constants::MAX_RESULTS;
use crate::fl;
use crate::icon_cache::icon_cache_handle;
use crate::message::Message;
use crate::operation::{Operation, OperationKind};
use crate::pages::{ContextPage, ExplorePage, NavPage};
use crate::search::SearchResult;
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
    if matches!(nav_page, NavPage::Applets) {
        if !sources.is_empty()
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
