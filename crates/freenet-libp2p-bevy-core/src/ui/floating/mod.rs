#[path = "__basic__/mod.rs"]
mod basic;

pub use basic::components::FloatingPopup;
pub use basic::constants::DEFAULT_POPUP_EDGE_MARGIN;
pub use basic::enums::{PopupAlign, PopupAnchor};
pub use basic::newtypes::{PopupEdgeMargin, PopupGap};

mod ui_plugin;
pub use ui_plugin::UiPlugin;

mod popup_placement;
pub use popup_placement::PopupPlacement;

mod popup_side;
pub use popup_side::PopupSide;

#[path = "__bevy_systems__/mod.rs"]
pub mod bevy_systems;
