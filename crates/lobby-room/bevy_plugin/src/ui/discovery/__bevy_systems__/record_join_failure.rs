use crate::ui;
use bevy::prelude::*;

use freenet_libp2p_bevy_core::replica_sync;
use ui::discovery::JoinError;

pub fn record_join_failure(
    mut failures: MessageReader<replica_sync::JoinFailure>,
    mut join_error: ResMut<JoinError>,
) {
    for failure in failures.read() {
        match failure {
            replica_sync::JoinFailure::ZoneGone(room) => {
                **join_error = Some(format!(
                    "Room {} no longer exists (all players left)",
                    room.as_str()
                ));
            }
            replica_sync::JoinFailure::HistoryUnavailable(_) => {
                **join_error = Some(String::from("Couldn't get room history"));
            }
        }
    }
}
