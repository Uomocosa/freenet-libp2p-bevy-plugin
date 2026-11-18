use bevy::prelude::Color;
use freenet_libp2p_bevy_plugin::lobby_room::{spectator, ui};

use crate::starboard;

/// Render layer shared by the lobby preview camera and its board. The in-room
/// board stays on layer 0, so neither camera ever draws the other's board.
pub const PREVIEW_LAYER: usize = 1;

/// The preview camera draws before the lobby camera, so the viewport image is
/// already filled when the UI samples it in the same frame.
pub const PREVIEW_CAMERA_ORDER: isize = -1;

/// Side of the preview board in target pixels: the 1024-cell board is fitted to
/// the 200 px short side of the 320 x 200 `RoomPreviewTarget` image.
pub const PREVIEW_BOARD_SIDE: f32 = 200.0;

/// Backdrop of the preview target: the lobby viewport's own dark colour, so the
/// letterbox around the board matches the panel the image sits in.
pub const PREVIEW_BACKDROP_COLOR: Color = ui::discovery::INPUT_COLOR;

/// Render layer of the streamed zone, separate from both the room and the
/// preview, so three cameras can each draw their own board.
pub const STREAM_LAYER: usize = 2;

/// The stream camera draws first: its 1280 x 800 target is the largest image
/// the UI samples in the frame.
pub const STREAM_CAMERA_ORDER: isize = -2;

/// Side of the streamed board in target pixels: the board is fitted to the
/// 800 px short side of the 16:10 `RoomStreamTarget` image, exactly like the
/// in-room board is fitted to the window.
pub const STREAM_BOARD_SIDE: f32 = 800.0;

/// Backdrop of the stream target: the same lobby viewport colour as the preview.
pub const STREAM_BACKDROP_COLOR: Color = ui::discovery::INPUT_COLOR;

/// Green of the own-brush accent, reused so a highlighted friend reads as "this one".
pub const FRIEND_HIGHLIGHT_RGBA: [u8; 4] = starboard::render::OWN_OUTLINE_RGBA;

/// Label colour of a highlighted friend, the own-accent label green.
pub const FRIEND_LABEL_COLOR: Color = starboard::render::OWN_LABEL_COLOR;

/// Fixed ring size in preview-target pixels: a default cursor is only about 6 px at
/// the 200 px preview scale, so a brush-shaped ring would vanish.
pub const FRIEND_HIGHLIGHT_RING_PX: u16 = 12;

/// Thickness of the ring band, in preview-target pixels.
pub const FRIEND_HIGHLIGHT_BAND_PX: u16 = 2;

/// Depth of a highlight sprite, above the board and the remote cursor sprites.
pub const FRIEND_HIGHLIGHT_Z: f32 = 2.0;

pub const PREVIEW_SLOT: starboard::preview::SlotRenderConfig =
    starboard::preview::SlotRenderConfig {
        slot: spectator::SpectateSlot::Preview,
        layer: PREVIEW_LAYER,
        camera_order: PREVIEW_CAMERA_ORDER,
        board_side: PREVIEW_BOARD_SIDE,
        backdrop: PREVIEW_BACKDROP_COLOR,
        placement: starboard::render::CursorPlacement::Preview,
        labels: starboard::render::CursorLabelKind::None,
    };

pub const STREAM_SLOT: starboard::preview::SlotRenderConfig =
    starboard::preview::SlotRenderConfig {
        slot: spectator::SpectateSlot::Stream,
        layer: STREAM_LAYER,
        camera_order: STREAM_CAMERA_ORDER,
        board_side: STREAM_BOARD_SIDE,
        backdrop: STREAM_BACKDROP_COLOR,
        placement: starboard::render::CursorPlacement::Stream,
        labels: starboard::render::CursorLabelKind::MemberName,
    };

pub const SLOT_RENDERS: [starboard::preview::SlotRenderConfig; 2] = [PREVIEW_SLOT, STREAM_SLOT];
