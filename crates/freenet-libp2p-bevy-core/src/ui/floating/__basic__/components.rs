use bevy::prelude::Component;

use crate::ui;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct FloatingPopup {
    pub anchor: ui::floating::PopupAnchor,
    pub preferred_side: ui::floating::PopupSide,
    pub cross_align: ui::floating::PopupAlign,
    pub gap: ui::floating::PopupGap,
    pub edge_margin: ui::floating::PopupEdgeMargin,
}

// no test_usage necessary
