use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::discovery;

#[must_use]
pub fn is_in_room(status: Res<discovery::Status>) -> bool {
    let status = status.into_inner();
    status
        .room
        .as_ref()
        .is_some_and(|room| room.phase == discovery::RoomPhase::Ready)
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::{discovery, identity};

    use super::is_in_room;

    #[derive(Resource, Default)]
    struct HasRun(bool);

    fn mark(mut has_run: ResMut<HasRun>) {
        has_run.0 = true;
    }

    fn status_with(phase: discovery::RoomPhase) -> discovery::Status {
        discovery::Status {
            room: Some(discovery::JoinedRoom {
                name: identity::RoomName::from("alpha"),
                created_at: discovery::UnixTime::from_secs(0),
                phase,
                members: discovery::Members::new(),
            }),
            ..discovery::Status::default()
        }
    }

    fn holds(status: discovery::Status) -> bool {
        let mut app = App::new();
        app.insert_resource(status);
        app.init_resource::<HasRun>();
        app.add_systems(Update, mark.run_if(is_in_room));
        app.update();
        app.world().resource::<HasRun>().0
    }

    #[test]
    fn test_usage() {
        assert!(!holds(discovery::Status::default()));
        assert!(!holds(status_with(discovery::RoomPhase::Joining)));
        assert!(holds(status_with(discovery::RoomPhase::Ready)));
    }
}
