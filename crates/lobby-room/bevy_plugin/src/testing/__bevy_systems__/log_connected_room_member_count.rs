use bevy::prelude::*;

use crate::discovery;
use crate::testing;

pub fn log_connected_room_member_count(
    state: Option<Res<discovery::state_machine::DiscoveryState>>,
    mut last_logged: Local<Option<testing::LoggedRoomMemberCount>>,
) {
    let Some(state) = state else {
        return;
    };
    if !state.is_changed() {
        return;
    }
    let Some(room) = state.room.as_ref() else {
        return;
    };
    let now = testing::LoggedRoomMemberCount {
        room: room.name.clone(),
        connected: state.connected_members().count(),
    };
    if last_logged.as_ref() == Some(&now) {
        return;
    }
    tracing::info!(
        "lobby mesh room={} connected={}",
        now.room.as_str(),
        now.connected
    );
    *last_logged = Some(now);
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::log_connected_room_member_count;
    use crate::discovery;
    use crate::identity;
    use crate::testing;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        let system = app
            .world_mut()
            .register_system(log_connected_room_member_count);
        assert!(app.world_mut().run_system(system).is_ok());
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .in_room(discovery::JoinedRoom {
                name: discovery::RoomName::from("alpha"),
                created_at: time::UnixTime::from_secs(1),
                phase: discovery::JoinPhase::Ready,
                members: std::collections::BTreeMap::new(),
                how_entered: discovery::HowRoomWasEntered::Created,
            })
            .build();
        app.insert_resource(state);
        assert!(app.world_mut().run_system(system).is_ok());
        assert!(app.world_mut().run_system(system).is_ok());
    }
}
