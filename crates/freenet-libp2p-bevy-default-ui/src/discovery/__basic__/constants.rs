use bevy::prelude::Color;

pub const ROOM_COLOR: Color = Color::srgb(0.12, 0.28, 0.38);
pub const CURRENT_ROOM_COLOR: Color = Color::srgb(0.18, 0.45, 0.25);

pub const PANEL_COLOR: Color = Color::srgb(0.30, 0.12, 0.34);
pub const BUTTON_COLOR: Color = Color::srgb(0.20, 0.24, 0.32);
pub const BUTTON_LABEL_COLOR: Color = Color::WHITE;
pub const DISABLED_BUTTON_COLOR: Color = Color::srgb(0.13, 0.13, 0.14);
pub const DISABLED_BUTTON_LABEL_COLOR: Color = Color::srgb(0.38, 0.38, 0.40);
pub const INPUT_COLOR: Color = Color::srgb(0.10, 0.10, 0.12);
pub const ACCENT_COLOR: Color = Color::srgb(0.38, 0.65, 0.98);

pub const DOOR_WIDTH: f32 = 44.0;
pub const DOOR_HEIGHT: f32 = 68.0;
pub const DOOR_SWING: f32 = 0.45;
pub const DOOR_OPEN_PER_SEC: f32 = 4.0;
pub const DOOR_FRAME_COLOR: Color = Color::srgb(0.25, 0.16, 0.10);
pub const DOOR_LIGHT_COLOR: Color = Color::srgb(0.96, 0.82, 0.48);
pub const DOOR_PANEL_COLOR: Color = Color::srgb(0.47, 0.29, 0.16);
pub const DOOR_KNOB_COLOR: Color = Color::srgb(0.90, 0.75, 0.30);
pub const PROMPT_BACKDROP_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.6);
pub const YES_COLOR: Color = Color::srgb(0.62, 0.20, 0.20);

pub const HOVER_PER_SEC: f32 = 8.0;
pub const HOVER_SCALE: f32 = 1.04;
pub const HOVER_OUTLINE_WIDTH: f32 = 2.0;
pub const CARET_COLOR: Color = Color::srgb(0.92, 0.94, 0.98);
pub const CARET_SELECTION_COLOR: Color = Color::srgba(0.38, 0.65, 0.98, 0.45);

pub const SPINNER_SIZE: f32 = 48.0;
pub const SPIN_TURNS_PER_SEC: f32 = 1.0;
pub const SPINNER_TRACK_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.08);
pub const CONNECTING_TEXT_COLOR: Color = Color::srgb(0.62, 0.66, 0.72);
