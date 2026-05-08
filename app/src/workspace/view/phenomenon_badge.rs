//! Reusable badge component for Phenomenon-styled launch modals.
use warp_core::ui::theme::{phenomenon::PhenomenonStyle, Fill};
use warpui::elements::{
    ConstrainedBox, Container, CornerRadius, CrossAxisAlignment, Flex, MainAxisSize, ParentElement,
    Radius, Text,
};
use warpui::Element;

use crate::appearance::Appearance;

/// Renders the "New" pill badge used in Phenomenon-styled launch modals.
///
/// Matches the design spec: 24px height, 8px horizontal padding, 14px font,
/// full-pill corner radius, with colors from `PhenomenonStyle`.
pub fn render_new_badge(appearance: &Appearance) -> Box<dyn Element> {
    let text = Text::new_inline("New".to_string(), appearance.ui_font_family(), 14.)
        .with_color(PhenomenonStyle::modal_badge_text())
        .finish();
    ConstrainedBox::new(
        Container::new(
            Flex::row()
                .with_cross_axis_alignment(CrossAxisAlignment::Center)
                .with_main_axis_size(MainAxisSize::Min)
                .with_child(text)
                .finish(),
        )
        .with_horizontal_padding(8.)
        .with_background(Fill::Solid(PhenomenonStyle::modal_badge_background()))
        .with_corner_radius(CornerRadius::with_all(Radius::Percentage(50.)))
        .finish(),
    )
    .with_height(24.)
    .finish()
}
