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

#[cfg(test)]
mod tests {
    use crate::ui;
    use bevy::prelude::*;

    use super::record_join_failure;
    use freenet_libp2p_bevy_core::identity;
    use freenet_libp2p_bevy_core::replica_sync;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_message::<replica_sync::JoinFailure>();
        app.init_resource::<ui::discovery::JoinError>();
        app.add_systems(Update, record_join_failure);
        app.world_mut()
            .resource_mut::<Messages<replica_sync::JoinFailure>>()
            .write(replica_sync::JoinFailure::ZoneGone(
                identity::ZoneName::from("alpha"),
            ));
        app.update();
        assert_eq!(
            **app.world().resource::<ui::discovery::JoinError>(),
            Some(String::from(
                "Room alpha no longer exists (all players left)"
            ))
        );
    }

    #[test]
    fn history_unavailable_sets_history_error() {
        let mut app = App::new();
        app.add_message::<replica_sync::JoinFailure>();
        app.init_resource::<ui::discovery::JoinError>();
        app.add_systems(Update, record_join_failure);
        app.world_mut()
            .resource_mut::<Messages<replica_sync::JoinFailure>>()
            .write(replica_sync::JoinFailure::HistoryUnavailable(
                identity::ZoneName::from("beta"),
            ));
        app.update();
        assert_eq!(
            **app.world().resource::<ui::discovery::JoinError>(),
            Some(String::from("Couldn't get room history"))
        );
    }
}
