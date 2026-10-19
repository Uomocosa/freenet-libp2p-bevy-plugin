use bevy::prelude::Color;

pub const BOARD_CHECKER_RGBA: [[u8; 4]; 2] = [[38, 40, 46, 255], [48, 50, 57, 255]];
pub const CHECKER_CELLS: usize = 8;
pub const BOARD_BORDER_COLOR: Color = Color::srgb_u8(70, 74, 84);
pub const BOARD_BORDER_PX: f32 = 1.0;
pub const PREVIEW_FILL_ALPHA: u8 = 128;
pub const OWN_PREVIEW_ALPHA: f32 = 0.5;
pub const OWN_CURSOR_Z: f32 = 2.0;
pub const OWN_OUTLINE_RGBA: [u8; 4] = [57, 255, 20, 255];
pub const OWN_OUTLINE_PX: u16 = 2;
pub const LABEL_FONT_SIZE: f32 = 12.0;
pub const LABEL_GAP_PX: f32 = 6.0;
pub const LABEL_Z: f32 = 0.5;
pub const LABEL_COLOR: Color = Color::WHITE;
pub const OWN_LABEL_COLOR: Color = Color::srgb_u8(57, 255, 20);
pub const LABEL_BACKGROUND: Color = Color::srgba(0.0, 0.0, 0.0, 0.6);
pub const OWN_FALLBACK_NAME: &str = "you";
