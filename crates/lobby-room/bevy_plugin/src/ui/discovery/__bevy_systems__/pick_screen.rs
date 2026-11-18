use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::*;

use core_discovery::state_machine::DiscoveryState;
use ui::discovery::{JoinError, Screen};

pub fn pick_screen(
    state: Option<Res<DiscoveryState>>,
    join_error: Option<Res<JoinError>>,
    screen: Res<State<Screen>>,
    mut next: ResMut<NextState<Screen>>,
) {
    let current = *screen.into_inner().get();
    let has_join_error = join_error.is_some_and(|err| err.is_some());
    let (phase, is_lobby_fetched) = state.map_or((None, false), |state| {
        let inner = state.into_inner();
        (
            inner.room.as_ref().map(|room| room.phase),
            inner.is_lobby_fetched,
        )
    });
    let wanted = if has_join_error {
        Screen::JoinFailed
    } else {
        match (phase, current) {
            (Some(core_discovery::JoinPhase::Ready), _) => Screen::InRoom,
            (Some(core_discovery::JoinPhase::Joining { .. }), _) => Screen::JoiningRoom,
            (None, Screen::Connecting) if !is_lobby_fetched => Screen::Connecting,
            (None, _) => Screen::Lobby,
        }
    };
    if wanted != current {
        tracing::info!(target: "discovery", "ui screen {current:?} -> {wanted:?}");
        next.set(wanted);
    }
}
