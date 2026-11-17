use bevy::prelude::Color;
use freenet_libp2p_bevy_plugin::lobby_room::spectator;

use crate::starboard;

pub struct SlotRenderConfig {
    pub slot: spectator::SpectateSlot,
    pub layer: usize,
    pub camera_order: isize,
    pub board_side: f32,
    pub backdrop: Color,
    pub placement: starboard::render::CursorPlacement,
    pub labels: starboard::render::CursorLabelKind,
}

// no test_usage necessary
