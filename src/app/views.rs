use std::cmp;
use std::collections::HashMap;

use cosmic::cosmic_theme;
use cosmic::iced::Length;
use cosmic::{Element, widget};

use crate::app_id::AppId;
use crate::app_info::WaylandCompatibility;
use crate::constants::MAX_RESULTS;
use crate::fl;
use crate::message::Message;
use crate::search::SearchResult;

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
