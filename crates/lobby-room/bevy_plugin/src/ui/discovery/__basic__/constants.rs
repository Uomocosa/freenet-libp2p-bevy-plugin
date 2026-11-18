use crate::ui;
use bevy::prelude::Color;

pub const ROOM_ROW_WIDTH: f32 = 232.0;
pub const ROOM_NAME_WIDTH: f32 = 128.0;
pub const ROOM_NAME_SCROLL_PER_SEC: f32 = 26.0;
pub const ROOM_BUBBLE_HAS_SPACE_COLOR: Color = Color::srgb(0.16, 0.44, 0.24);
pub const ROOM_BUBBLE_EMPTY_COLOR: Color = Color::srgb(0.30, 0.31, 0.35);
pub const ROOM_BUBBLE_FULL_COLOR: Color = Color::srgb(0.58, 0.20, 0.20);
pub const FRIEND_BADGE_COLOR: Color = Color::srgb(0.35, 0.80, 0.44);
pub const FRIEND_DIVIDER_WIDTH: f32 = 1.0;
pub const FRIEND_DIVIDER_HEIGHT: f32 = 10.0;
pub const FRIEND_DIVIDER_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.60);

pub const OCCUPANTS_SLOT: usize = 0;
pub const ROOM_SIZE_SLOT: usize = 1;
pub const FRIENDS_SLOT: usize = 2;

/// Bevy's default line height is 1.2 of the font size; keeping it here holds the pill at its current height.
pub const ROOM_NUMBER_ROW_HEIGHT: f32 = 13.2;
pub const FRIEND_NUMBER_ROW_HEIGHT: f32 = 10.8;

pub const ROOM_NUMBER_STYLE: ui::roll::NumberStyle = ui::roll::NumberStyle {
    font_size: 11.0,
    row_height: ROOM_NUMBER_ROW_HEIGHT,
    text_color: BUTTON_LABEL_COLOR,
};

pub const FRIEND_NUMBER_STYLE: ui::roll::NumberStyle = ui::roll::NumberStyle {
    font_size: 9.0,
    row_height: FRIEND_NUMBER_ROW_HEIGHT,
    text_color: BUTTON_LABEL_COLOR,
};
pub const ROOM_BUBBLE_TOOLTIP_FONT_SIZE: f32 = 10.0;
pub const ROOM_BUBBLE_TOOLTIP_Z: i32 = 60;
pub const ROOM_BUBBLE_TOOLTIP_BG_COLOR: Color = Color::srgb(0.18, 0.07, 0.21);

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
pub const CREATE_PRESS_SCALE: f32 = 0.94;
pub const CREATE_SHAKE_SECS: f32 = 0.25;
pub const CREATE_SHAKE_PX: f32 = 4.0;
pub const CREATE_SHAKE_TURNS: f32 = 2.0;
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
pub const PREVIEW_Z: i32 = 20;
pub const PREVIEW_RADIUS: f32 = 12.0;
pub const PREVIEW_INNER_RADIUS: f32 = 8.0;
pub const PREVIEW_FRAME_WIDTH: f32 = 1.0;
pub const PREVIEW_FRAME_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.10);
pub const PREVIEW_PLACEHOLDER_SPINNER_SIZE: f32 = 28.0;
pub const PREVIEW_PLACEHOLDER_SPINNER_WIDTH: f32 = 3.0;
pub const LOBBY_PANEL_RIGHT: f32 = 12.0;
pub const LOBBY_PANEL_WIDTH: f32 = 248.0;
pub const LOBBY_PANEL_GAP: f32 = 12.0;
pub const PREVIEW_LIST_GAP: f32 = 16.0;
pub const PREVIEW_RIGHT: f32 = LOBBY_PANEL_RIGHT + LOBBY_PANEL_WIDTH + PREVIEW_LIST_GAP;
pub const DOUBLE_CLICK_SECS: f32 = 0.35;
pub const PREVIEW_PADDING: f32 = 8.0;
pub const PREVIEW_ROW_GAP: f32 = 8.0;
pub const PREVIEW_MEMBER_GAP: f32 = 3.0;
pub const PREVIEW_TITLE_FONT_SIZE: f32 = 15.0;
pub const PREVIEW_MEMBER_FONT_SIZE: f32 = 11.0;
pub const PREVIEW_VIEWPORT_PADDING: f32 = 8.0;
pub const PREVIEW_FRIENDS_MAX: usize = 8;
pub const PREVIEW_ACTION_INSET: f32 = 8.0;
pub const PREVIEW_ACTION_RESERVE: f32 = 40.0;
pub const PREVIEW_BUTTON_FONT_SIZE: f32 = 14.0;
pub const PREVIEW_BUTTON_PADDING_H: f32 = 14.0;
pub const PREVIEW_BUTTON_PADDING_V: f32 = 6.0;
pub const ENTER_BUTTON_COLOR: Color = Color::srgb(0.16, 0.52, 0.28);
pub const ROOM_FULL_BUTTON_COLOR: Color = Color::srgb(0.30, 0.31, 0.35);
pub const ROOM_FULL_BUTTON_LABEL_COLOR: Color = Color::srgb(0.70, 0.71, 0.74);
pub const SELECTED_ROOM_PREVIEW_OUTLINE_COLOR: Color = Color::srgba(0.38, 0.65, 0.98, 0.55);
pub const STREAMED_ROOM_OUTLINE_COLOR: Color = ACCENT_COLOR;
pub const STREAMED_ROOM_OUTLINE_WIDTH: f32 = 3.0;
pub const STREAMED_ROOM_BG_COLOR: Color = Color::srgb(0.18, 0.38, 0.50);

pub const SPINNER_SIZE: f32 = 48.0;
pub const SPIN_TURNS_PER_SEC: f32 = 1.0;
pub const SPINNER_TRACK_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.08);
pub const CONNECTING_TEXT_COLOR: Color = Color::srgb(0.62, 0.66, 0.72);

pub const STREAM_TOP: f32 = 48.0;
pub const STREAM_LEFT: f32 = 12.0;
pub const STREAM_BOTTOM: f32 = 12.0;
pub const STREAM_LOBBY_GAP: f32 = 16.0;
pub const STREAM_RIGHT: f32 = LOBBY_PANEL_RIGHT + LOBBY_PANEL_WIDTH + STREAM_LOBBY_GAP;
pub const STREAM_Z: i32 = 10;
pub const STREAM_RADIUS: f32 = 12.0;
pub const STREAM_FRAME_PADDING: f32 = 4.0;
pub const STREAM_VIEWPORT_RADIUS: f32 = 8.0;
pub const STREAM_IMAGE_PADDING: f32 = 12.0;
pub const STREAM_IMAGE_TOP_INSET: f32 = 64.0;
pub const STREAM_IMAGE_ASPECT: f32 = 1.6;
pub const STREAM_HEADER_INSET: f32 = 12.0;
pub const STREAM_HEADER_GAP: f32 = 8.0;
pub const STREAM_HEADER_PADDING_H: f32 = 10.0;
pub const STREAM_HEADER_PADDING_V: f32 = 6.0;
pub const STREAM_HEADER_RADIUS: f32 = 10.0;
pub const STREAM_CONTROL_BAR_COLOR: Color = Color::srgba(0.09, 0.10, 0.14, 0.88);
pub const STREAM_TITLE_FONT_SIZE: f32 = 15.0;
pub const STREAM_OCCUPANCY_FONT_SIZE: f32 = 11.0;
pub const STREAM_OCCUPANCY_PADDING_H: f32 = 7.0;
pub const STREAM_OCCUPANCY_PADDING_V: f32 = 2.5;
pub const STREAM_CONTROL_SIDE: f32 = 28.0;
pub const STREAM_CONTROL_RADIUS: f32 = 8.0;
pub const STREAM_ICON_SIDE: f32 = 14.0;
pub const STREAM_ICON_STROKE: f32 = 2.0;
pub const STREAM_CLOSE_ICON_TOP: f32 = 6.0;
pub const STREAM_ICON_CORNER: f32 = 5.0;
pub const STREAM_PLACEHOLDER_SPINNER_SIZE: f32 = 28.0;
pub const STREAM_PLACEHOLDER_SPINNER_WIDTH: f32 = 3.0;
