use crate::ui;
use bevy::prelude::Color;

pub const ROSTER_BUBBLE_Z: i32 = 90;
pub const ROSTER_BUBBLE_ROOM_BAR_CLEARANCE: f32 = 52.0;
pub const ROSTER_BUBBLE_LEFT: f32 = 16.0;
pub const ROSTER_DOT_SIZE: f32 = 8.0;
pub const ROSTER_ROW_ICON_SIDE: f32 = 16.0;
pub const ROSTER_COUNT_FONT_SIZE: f32 = 11.0;
pub const ROSTER_NAME_FONT_SIZE: f32 = 12.0;
pub const ROSTER_YOU_FONT_SIZE: f32 = 10.0;
pub const ROSTER_TOOLTIP_FONT_SIZE: f32 = 10.0;
pub const ROSTER_LIST_WIDTH: f32 = 178.0;

pub const ROSTER_CONNECTED_COLOR: Color = Color::srgb(0.30, 0.82, 0.45);
pub const ROSTER_CONNECTING_COLOR: Color = Color::srgb(0.87, 0.71, 0.22);
pub const ROSTER_DISCONNECTED_COLOR: Color = Color::srgb(0.42, 0.44, 0.50);
pub const ROSTER_ROW_COLOR: Color = Color::srgb(0.15, 0.17, 0.22);
pub const ROSTER_LABEL_COLOR: Color = Color::WHITE;
pub const ROSTER_TOOLTIP_TEXT_COLOR: Color = ui::constants::DIM_TEXT_COLOR;
