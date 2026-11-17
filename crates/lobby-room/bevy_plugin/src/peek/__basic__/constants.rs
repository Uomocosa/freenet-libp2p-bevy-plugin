/// Render-target size for the room preview image; mirrors the viewport's
/// `PREVIEW_WIDTH` x `PREVIEW_HEIGHT` (320 x 200) in `ui::discovery`.
pub const PREVIEW_TARGET_WIDTH: u32 = 320;
pub const PREVIEW_TARGET_HEIGHT: u32 = 200;
pub const PREVIEW_STOP_LINGER_SECS: f32 = 1.0;
/// Render-target size for the streamed room; keeps the board's 16:10 aspect
/// (the step-4 panel will size itself from this).
pub const STREAM_TARGET_WIDTH: u32 = 1280;
pub const STREAM_TARGET_HEIGHT: u32 = 800;
