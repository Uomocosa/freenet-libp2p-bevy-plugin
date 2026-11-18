use crate::discovery as core_discovery;
use crate::ui;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;

use ui::discovery::{PREVIEW_HOVER_DELAY_SECS, RoomButton, SelectedRoom};

pub fn track_room_preview(
    time: Res<Time>,
    buttons: Query<(&RoomButton, &Hovered)>,
    mut selection: ResMut<SelectedRoom>,
    mut hovered_room: Local<Option<core_discovery::RoomName>>,
    mut hovered_secs: Local<f32>,
) {
    let delta_secs = time.into_inner().delta_secs();
    let hovered_now = buttons
        .iter()
        .find(|(_, hovered)| hovered.get())
        .map(|(button, _)| (**button).clone());
    if *hovered_room != hovered_now {
        *hovered_room = hovered_now;
        *hovered_secs = 0.0;
    } else if hovered_room.is_some() {
        *hovered_secs += delta_secs;
    }
    let Some(room) = hovered_room.as_ref() else {
        **selection = None;
        return;
    };
    if *hovered_secs >= PREVIEW_HOVER_DELAY_SECS {
        **selection = Some(room.clone());
    }
}
