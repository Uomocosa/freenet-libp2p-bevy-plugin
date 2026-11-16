use bevy::prelude::Color;

pub const ROOM_ROW_WIDTH: f32 = 212.0;
pub const ROOM_NAME_WIDTH: f32 = 128.0;
pub const ROOM_NAME_SCROLL_PER_SEC: f32 = 26.0;
pub const ROOM_BUBBLE_HAS_SPACE_COLOR: Color = Color::srgb(0.16, 0.44, 0.24);
pub const ROOM_BUBBLE_EMPTY_COLOR: Color = Color::srgb(0.30, 0.31, 0.35);
pub const ROOM_BUBBLE_FULL_COLOR: Color = Color::srgb(0.58, 0.20, 0.20);
pub const FRIEND_BADGE_COLOR: Color = Color::srgb(0.35, 0.80, 0.44);
pub const FRIEND_BADGE_BG_COLOR: Color = Color::srgb(0.09, 0.12, 0.10);

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

pub const MENU_Z: i32 = 110;
pub const MENU_WIDTH: f32 = 240.0;
pub const MENU_BACKDROP_ALPHA: f32 = 0.55;
pub const MENU_PANEL_RADIUS: f32 = 12.0;
pub const MENU_BUTTON_RADIUS: f32 = 8.0;
pub const MENU_LABEL_FONT_SIZE: f32 = 13.0;
pub const MENU_HINT_FONT_SIZE: f32 = 10.0;
pub const MENU_FRIENDS_BEAT_SECS: f32 = 0.12;

pub const HOVER_PER_SEC: f32 = 8.0;
pub const HOVER_SCALE: f32 = 1.04;
pub const HOVER_OUTLINE_WIDTH: f32 = 2.0;
pub const CARET_COLOR: Color = Color::srgb(0.92, 0.94, 0.98);
pub const CARET_SELECTION_COLOR: Color = Color::srgba(0.38, 0.65, 0.98, 0.45);

pub const ROOM_SEARCH_LABEL_FONT_SIZE: f32 = 12.0;
pub const ROOM_SEARCH_SPINNER_SIZE: f32 = 12.0;
pub const ROOM_SEARCH_TOOLTIP_FONT_SIZE: f32 = 10.0;
pub const ROOM_SEARCH_TOOLTIP_Z: i32 = 50;
pub const ROOM_SEARCH_TOOLTIP_BG_COLOR: Color = Color::srgb(0.18, 0.07, 0.21);

pub const PREVIEW_HOVER_DELAY_SECS: f32 = 0.3;
pub const PREVIEW_WIDTH: f32 = 320.0;
pub const PREVIEW_HEIGHT: f32 = 200.0;
pub const PREVIEW_TOP: f32 = 48.0;
/// Right edge of the lobby Root (12) plus its width (212 row + 16 padding) plus an 8 px gap.
pub const PREVIEW_RIGHT: f32 = 248.0;
pub const PREVIEW_PADDING: f32 = 8.0;
pub const PREVIEW_ROW_GAP: f32 = 8.0;
pub const PREVIEW_MEMBER_GAP: f32 = 3.0;
pub const PREVIEW_TITLE_FONT_SIZE: f32 = 15.0;
pub const PREVIEW_MEMBER_FONT_SIZE: f32 = 11.0;
pub const PREVIEW_VIEWPORT_PADDING: f32 = 6.0;
pub const PREVIEW_BUTTON_RADIUS: f32 = 6.0;
pub const ENTER_BUTTON_COLOR: Color = Color::srgb(0.16, 0.52, 0.28);
pub const ROOM_FULL_BUTTON_COLOR: Color = Color::srgb(0.30, 0.31, 0.35);
pub const ROOM_FULL_BUTTON_LABEL_COLOR: Color = Color::srgb(0.70, 0.71, 0.74);

pub const SPINNER_SIZE: f32 = 48.0;
pub const SPIN_TURNS_PER_SEC: f32 = 1.0;
pub const SPINNER_TRACK_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.08);
pub const CONNECTING_TEXT_COLOR: Color = Color::srgb(0.62, 0.66, 0.72);
