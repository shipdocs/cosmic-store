//! Search-related types and functionality

use cosmic::iced::{Alignment, Length};
use cosmic::widget;
use cosmic::Element;
use cosmic::cosmic_theme;
use std::sync::Arc;

use crate::app_id::AppId;
use crate::app_info::AppInfo;
use crate::constants::ICON_SIZE_SEARCH;
use crate::editors_choice::EDITORS_CHOICE;
use crate::ui::badges::wayland_compat_badge;
use crate::ui::cards::styled_icon;
use crate::ui::GridMetrics;
use crate::utils::format_download_count;
use crate::icon_cache::icon_cache_handle;

// Import Message type and fl macro from main
pub use crate::{fl, Message};

/// Search result sorting mode
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SearchSortMode {
    Relevance,
    MostDownloads,
    RecentlyUpdated,
    BestWaylandSupport,
}

/// Wayland compatibility filter mode
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WaylandFilter {
    All,
    Excellent,  // Low risk
    Good,       // Medium risk
    Caution,    // High risk
    Limited,    // Critical risk
    Unknown,
}

/// A search result from a backend
#[derive(Clone, Debug)]
pub struct SearchResult {
    backend_name: &'static str,
    pub id: AppId,
    pub icon_opt: Option<widget::icon::Handle>,
    // Info from selected source
    pub info: Arc<AppInfo>,
    /// Weight for sorting search results (higher = better match)
    pub weight: i64,
}

impl SearchResult {
    /// Create a new search result
    pub fn new(
        backend_name: &'static str,
        id: AppId,
        icon_opt: Option<widget::icon::Handle>,
        info: Arc<AppInfo>,
        weight: i64,
    ) -> Self {
        Self {
            backend_name,
            id,
            icon_opt,
            info,
            weight,
        }
    }

    /// Get the backend name for this search result
    pub fn backend_name(&self) -> &'static str {
        self.backend_name
    }

    /// Calculate grid metrics for displaying search results
    pub fn grid_metrics(spacing: &cosmic_theme::Spacing, width: usize) -> GridMetrics {
        GridMetrics::new(width, 240 + 2 * spacing.space_s as usize, spacing.space_xxs)
    }

    /// Create a grid view of search results
    ///
    /// # Arguments
    /// * `results` - Slice of search results to display
    /// * `spacing` - Cosmic theme spacing values
    /// * `width` - Available width for the grid
    /// * `callback` - Function to create a message when a result is clicked
    pub fn grid_view<'a, F: Fn(usize) -> Message + 'a>(
        results: &'a [Self],
        spacing: cosmic_theme::Spacing,
        width: usize,
        callback: F,
    ) -> Element<'a, Message> {
        let GridMetrics {
            cols,
            item_width,
            column_spacing,
        } = Self::grid_metrics(&spacing, width);

        let mut grid = widget::grid();
        let mut col = 0;
        for (result_i, result) in results.iter().enumerate() {
            if col >= cols {
                grid = grid.insert_row();
                col = 0;
            }
            grid = grid.push(
                widget::mouse_area(result.card_view(&spacing, item_width))
                    .on_press(callback(result_i)),
            );
            col += 1;
        }
        grid.column_spacing(column_spacing)
            .row_spacing(column_spacing)
            .into()
    }

    /// Create a card view for this search result
    pub fn card_view<'a>(
        &'a self,
        spacing: &cosmic_theme::Spacing,
        width: usize,
    ) -> Element<'a, Message> {
        use cosmic::theme;
        use cosmic::widget;

        // Check for editor's choice and verified status
        let is_editors_choice = EDITORS_CHOICE
            .iter()
            .any(|choice_id| choice_id == &self.id.normalized());
        let is_verified = self.info.verified;

        // Always show a compatibility badge - every app gets a status indicator
        let compat_badge = wayland_compat_badge(&self.info, 16);

        let mut name_row = vec![];
        name_row.push(widget::text::body(&self.info.name)
            .height(Length::Fixed(20.0))
            .into());

        if let Some(badge) = compat_badge {
            name_row.push(badge.into());
        }


        widget::container(
            widget::row::with_children(vec![
                match &self.icon_opt {
                    Some(icon) => styled_icon(icon.clone(), ICON_SIZE_SEARCH),
                    None => {
                        widget::Space::with_width(Length::Fixed(ICON_SIZE_SEARCH as f32)).into()
                    }
                },
                widget::column::with_children(vec![
                    widget::row::with_children(name_row)
                        .spacing(spacing.space_xxs)
                        .into(),
                    widget::text::caption(&self.info.summary)
                        .height(Length::Fixed(28.0))
                        .into(),
                    widget::row::with_children(vec![
                        if self.info.source_id == "flathub" && self.info.monthly_downloads > 0 {
                            widget::tooltip(
                                widget::text::caption(format_download_count(self.info.monthly_downloads)),
                                widget::text(fl!("monthly-downloads-tooltip")),
                                widget::tooltip::Position::Bottom,
                            )
                            .into()
                        } else {
                            widget::Space::with_width(Length::Fixed(0.0)).into()
                        },
                        widget::horizontal_space().into(),
                        if is_editors_choice {
                            widget::tooltip(
                                widget::icon::icon(icon_cache_handle("starred-symbolic", 16))
                                    .size(16),
                                widget::text(fl!("editors-choice-tooltip")),
                                widget::tooltip::Position::Bottom,
                            )
                            .into()
                        } else if is_verified {
                            widget::tooltip(
                                widget::icon::icon(icon_cache_handle("checkmark-symbolic", 16))
                                    .size(16),
                                widget::text(fl!("verified-tooltip")),
                                widget::tooltip::Position::Bottom,
                            )
                            .into()
                        } else {
                            widget::Space::with_width(Length::Fixed(0.0)).into()
                        },
                    ])
                    .spacing(spacing.space_xxs)
                    .align_y(Alignment::Center)
                    .into()
                ])
                .into(),
            ])
            .align_y(Alignment::Center)
            .spacing(spacing.space_s),
        )
        .align_y(Alignment::Center)
        .width(Length::Fixed(width as f32))
        .height(Length::Fixed(64.0 + (spacing.space_xxs as f32) * 2.0))
        .padding([spacing.space_xxs, spacing.space_s])
        .class(theme::Container::Card)
        .into()
    }
}
