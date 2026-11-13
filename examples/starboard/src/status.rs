use bevy::prelude::*;
use discovery::state_machine::DiscoveryState;
use freenet_libp2p_bevy_plugin::lobby_room::discovery;

use crate::starboard;

pub fn log_canvas_hash(
    time: Res<Time>,
    state: Option<Res<DiscoveryState>>,
    canvas: Res<starboard::canvas::Canvas>,
    cursor: Res<starboard::input::CursorCell>,
    brush: Res<starboard::input::OwnBrush>,
    mut accumulator: Local<f32>,
) {
    let time = time.into_inner();
    let Some(state) = state.map(Res::into_inner) else {
        return;
    };
    let canvas = canvas.into_inner();
    let cursor = cursor.into_inner();
    let brush = brush.into_inner();
    *accumulator += time.delta_secs();
    if *accumulator < 1.0 {
        return;
    }
    *accumulator = 0.0;
    let Some(room) = state.room.as_ref() else {
        return;
    };
    let cell = cursor.unwrap_or_default();
    let held = starboard::canvas::read_cell(canvas, cell);
    let tiles = canvas.tiles.iter().filter(|tile| tile.is_some()).count();
    let percent = starboard::canvas::SIZE_PERCENTS
        .get(usize::from(brush.size_index))
        .copied()
        .unwrap_or(100);
    tracing::info!(
        "starboard canvas room={} hash={} tiles={tiles} cursor={},{} color={:?} size={percent}%",
        room.name.as_str(),
        starboard::canvas::hash_canvas(canvas),
        cell.x,
        cell.y,
        held.top.color
    );
}
