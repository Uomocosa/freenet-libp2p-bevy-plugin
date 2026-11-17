use bevy::prelude::Color;

/// Render layer shared by the lobby preview camera and its board. The in-room
/// board stays on layer 0, so neither camera ever draws the other's board.
pub const PREVIEW_LAYER: usize = 1;

/// The preview camera draws before the lobby camera, so the viewport image is
/// already filled when the UI samples it in the same frame.
pub const PREVIEW_CAMERA_ORDER: isize = -1;

/// Side of the preview board in target pixels: the 1024-cell board is fitted to
/// the 200 px short side of the 320 x 200 `RoomPreviewTarget` image.
pub const PREVIEW_BOARD_SIDE: f32 = 200.0;

/// Backdrop of the preview target. The viewport is its own screen, so it keeps a
/// fixed dark backdrop instead of inheriting the window's `ClearColor`.
pub const PREVIEW_BACKDROP_COLOR: Color = Color::srgb(0.02, 0.02, 0.03);
