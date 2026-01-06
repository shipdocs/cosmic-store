//! Card rendering components for packages and search results

use cosmic::Element;
use cosmic::cosmic_theme;
use cosmic::iced::{Alignment, Border, Color, Length};
use cosmic::theme;
use cosmic::widget;
use std::collections::HashMap;

use crate::app_id::AppId;
use crate::app_info::{AppInfo, WaylandCompatibility};
use crate::constants::ICON_SIZE_PACKAGE;
use crate::ui::badges::wayland_compat_badge;

// Import Message type from main
pub use crate::Message;

/// Create a styled icon container with rounded corners
pub fn styled_icon<'a>(icon: widget::icon::Handle, size: u16) -> Element<'a, Message> {
    widget::container(widget::icon::icon(icon).size(size))
        .padding(8)
        .class(theme::Container::custom(move |theme| {
            let cosmic = theme.cosmic();

            // Use a slightly elevated background for better contrast
            let base_color = cosmic.background.component.base;
            let bg_color = Color::from_rgba(
                (base_color.red + 0.05).min(1.0),
                (base_color.green + 0.05).min(1.0),
                (base_color.blue + 0.05).min(1.0),
                base_color.alpha,
            );

            widget::container::Style {
                icon_color: Some(cosmic.on_bg_color().into()),
                text_color: Some(cosmic.on_bg_color().into()),
                background: Some(bg_color.into()),
                border: Border {
                    radius: ((size + 16) as f32 * 0.25).into(), // Larger radius accounting for padding
                    width: 1.0,
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.05),
                },
                shadow: cosmic::iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.15),
                    offset: cosmic::iced::Vector::new(0.0, 3.0),
                    blur_radius: 8.0,
                },
            }
        }))
        .into()
}

/// Create a package card view
pub fn package_card_view<'a>(
    info: &'a AppInfo,
    icon_opt: Option<&'a widget::icon::Handle>,
    controls: Vec<Element<'a, Message>>,
    top_controls: Option<Vec<Element<'a, Message>>>,
    spacing: &cosmic_theme::Spacing,
    width: usize,
    app_stats: &'a HashMap<AppId, (u64, Option<WaylandCompatibility>)>,
) -> Element<'a, Message> {
    // Only show compatibility badge for Flathub apps (only they have the data)
    let compat_badge = if info.source_id == "flathub" {
        wayland_compat_badge(info, 16, app_stats)
    } else {
        None
    };

    let mut name_row = vec![
        widget::text::body(&info.name)
            .height(20.0)
            .width(width as f32 - 180.0)
            .into(),
    ];

    if let Some(badge) = compat_badge {
        name_row.push(badge);
    }

    let height = 20.0 + 28.0 + 32.0 + 3.0 * spacing.space_xxs as f32;
    let top_row_cap = 1 + top_controls
        .as_deref()
        .map(|elements| 1 + elements.len())
        .unwrap_or_default();
    let column = widget::column::with_children(vec![
        widget::row::with_capacity(top_row_cap)
            .push(widget::column::with_children(vec![
                widget::row::with_children(name_row)
                    .spacing(spacing.space_xxs)
                    .into(),
                widget::text::caption(&info.summary)
                    .height(28.0)
                    .width(width as f32 - 180.0)
                    .into(),
            ]))
            .push_maybe(top_controls.is_some().then_some(widget::horizontal_space()))
            .extend(top_controls.unwrap_or_default())
            .into(),
        widget::Space::with_height(Length::Fixed(spacing.space_xxs.into())).into(),
        widget::row::with_children(controls)
            .height(32.0)
            .spacing(spacing.space_xs)
            .into(),
    ]);

    let icon: Element<_> = match icon_opt {
        Some(icon) => styled_icon(icon.clone(), ICON_SIZE_PACKAGE),
        None => widget::Space::with_width(ICON_SIZE_PACKAGE as f32).into(),
    };

    widget::container(
        widget::row()
            .push(icon)
            .push(column)
            .align_y(Alignment::Center)
            .spacing(spacing.space_s),
    )
    .align_y(Alignment::Center)
    .width(width as f32)
    .height(height)
    .padding([spacing.space_xxs, spacing.space_s])
    .class(theme::Container::Card)
    .into()
}
