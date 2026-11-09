use bevy::prelude::*;

use crate::discovery;
use discovery::JoinError;
use freenet_libp2p_bevy_core::room_sync;

pub fn record_join_failure(
    mut failures: MessageReader<room_sync::JoinFailure>,
    mut join_error: ResMut<JoinError>,
) {
    for failure in failures.read() {
        match failure {
            room_sync::JoinFailure::RoomGone(room) => {
                **join_error = Some(format!(
                    "Room {} no longer exists (all players left)",
                    room.as_str()
                ));
            }
            room_sync::JoinFailure::HistoryUnavailable(_) => {
                **join_error = Some(String::from("Couldn't get room history"));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::discovery;
    use bevy::prelude::*;

    use super::record_join_failure;
    use freenet_libp2p_bevy_core::identity;
    use freenet_libp2p_bevy_core::room_sync;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_message::<room_sync::JoinFailure>();
        app.init_resource::<discovery::JoinError>();
        app.add_systems(Update, record_join_failure);
        app.world_mut()
            .resource_mut::<Messages<room_sync::JoinFailure>>()
            .write(room_sync::JoinFailure::RoomGone(identity::RoomName::from(
                "alpha",
            )));
        app.update();
        assert_eq!(
            **app.world().resource::<discovery::JoinError>(),
            Some(String::from(
                "Room alpha no longer exists (all players left)"
            ))
        );
    }

    #[test]
    fn history_unavailable_sets_history_error() {
        let mut app = App::new();
        app.add_message::<room_sync::JoinFailure>();
        app.init_resource::<discovery::JoinError>();
        app.add_systems(Update, record_join_failure);
        app.world_mut()
            .resource_mut::<Messages<room_sync::JoinFailure>>()
            .write(room_sync::JoinFailure::HistoryUnavailable(
                identity::RoomName::from("beta"),
            ));
        app.update();
        assert_eq!(
            **app.world().resource::<discovery::JoinError>(),
            Some(String::from("Couldn't get room history"))
        );
    }
}
