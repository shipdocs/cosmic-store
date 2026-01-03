//! Badge components for displaying compatibility and status indicators

use cosmic::iced::Color;
use cosmic::widget;
use cosmic::Element;

use crate::app_info::AppInfo;
use crate::app_info::{RiskLevel, WaylandSupport};
use crate::icon_cache::icon_cache_handle;

// Import Message type and fl macro from main
pub use crate::{fl, Message};

/// Helper function to create a styled badge icon
fn styled_badge_icon<'a>(
    icon_name: &'static str,
    icon_size: u16,
    icon_color: Color,
    _bg_color: Color,
) -> Element<'a, Message> {
    widget::icon::icon(icon_cache_handle(icon_name, icon_size))
        .size(icon_size)
        .class(cosmic::theme::Svg::Custom(std::rc::Rc::new(move |_theme| {
            cosmic::iced::widget::svg::Style {
                color: Some(icon_color),
            }
        })))
        .into()
}

/// Create a Wayland compatibility badge for an app
///
/// Shows a visual indicator of how well an app supports Wayland,
/// with appropriate colors and tooltips explaining the support level.
pub fn wayland_compat_badge<'a>(info: &'a AppInfo, icon_size: u16) -> Option<Element<'a, Message>> {
    let compat_badge = if let Some(compat) = info.wayland_compat_lazy() {
        match compat.risk_level {
            RiskLevel::Low => {
                Some(widget::tooltip(
                    styled_badge_icon(
                        "checkbox-checked-symbolic",
                        icon_size,
                        Color::from_rgb(0.15, 0.75, 0.3),
                        Color::from_rgba(0.15, 0.75, 0.3, 0.2),
                    ),
                    widget::text::caption(fl!("wayland-native-tooltip")),
                    widget::tooltip::Position::Bottom,
                ))
            }
            RiskLevel::Medium => {
                Some(widget::tooltip(
                    styled_badge_icon(
                        "dialog-information-symbolic",
                        icon_size,
                        Color::from_rgb(0.2, 0.6, 0.85),
                        Color::from_rgba(0.2, 0.6, 0.85, 0.2),
                    ),
                    widget::text::caption(format!("{:?} - Good Wayland support", compat.framework)),
                    widget::tooltip::Position::Bottom,
                ))
            }
            RiskLevel::High => {
                let tooltip_text = if matches!(compat.support, WaylandSupport::X11Only) {
                    fl!("x11-only-tooltip")
                } else {
                    fl!("wayland-issues-warning")
                };

                Some(widget::tooltip(
                    styled_badge_icon(
                        "dialog-warning-symbolic",
                        icon_size,
                        Color::from_rgb(1.0, 0.55, 0.0),
                        Color::from_rgba(1.0, 0.55, 0.0, 0.2),
                    ),
                    widget::text::caption(tooltip_text),
                    widget::tooltip::Position::Bottom,
                ))
            }
            RiskLevel::Critical => {
                let tooltip_text = fl!("x11-only-tooltip");
                Some(widget::tooltip(
                    styled_badge_icon(
                        "dialog-warning-symbolic",
                        icon_size,
                        Color::from_rgb(1.0, 0.3, 0.3),
                        Color::from_rgba(1.0, 0.3, 0.3, 0.2),
                    ),
                    widget::text::caption(tooltip_text),
                    widget::tooltip::Position::Bottom,
                ))
            }
        }
    } else {
        Some(widget::tooltip(
            styled_badge_icon(
                "dialog-question-symbolic",
                icon_size,
                Color::from_rgb(0.5, 0.5, 0.5),
                Color::from_rgba(0.5, 0.5, 0.5, 0.15),
            ),
            widget::text::caption("Wayland compatibility unknown"),
            widget::tooltip::Position::Bottom,
        ))
    };

    compat_badge.map(|badge| badge.into())
}
