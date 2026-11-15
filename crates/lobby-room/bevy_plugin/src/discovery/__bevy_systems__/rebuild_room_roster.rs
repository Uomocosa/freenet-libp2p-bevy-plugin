use bevy::prelude::*;

use crate::discovery;
use crate::p2p;

pub fn rebuild_room_roster(
    state: Option<Res<discovery::state_machine::DiscoveryState>>,
    ready: Option<Res<p2p::ReadyChannel>>,
    mut roster: ResMut<discovery::RoomRoster>,
) {
    let is_swarm_ready = ready.is_some_and(|ready| ready.ready_rx.borrow().is_some());
    *roster = state
        .map(|state| state.into_inner().room_roster(is_swarm_ready))
        .unwrap_or_default();
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::rebuild_room_roster;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;
    use crate::testing;
    use crate::time;

    fn app_with(
        state: discovery::state_machine::DiscoveryState,
        ready: Option<p2p::ReadyChannel>,
    ) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(state);
        app.init_resource::<discovery::RoomRoster>();
        if let Some(ready) = ready {
            app.insert_resource(ready);
        }
        app.add_systems(Update, rebuild_room_roster);
        app
    }

    fn joined() -> discovery::JoinedRoom {
        discovery::JoinedRoom {
            name: identity::RoomName::from("alpha"),
            created_at: time::UnixTime::EPOCH,
            phase: discovery::JoinPhase::Ready,
            members: discovery::Members::new(),
            how_entered: discovery::HowRoomWasEntered::Created,
        }
    }

    #[test]
    fn test_usage() {
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .in_room(joined())
            .build();
        let ready = p2p::ReadyChannel::default();
        ready.ready_tx.send_replace(Some(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        }));
        let mut app = app_with(state, Some(ready));
        app.update();
        let roster = app.world().resource::<discovery::RoomRoster>();
        assert_eq!(roster.own, discovery::OwnRoomStatus::Connected);
        assert_eq!(roster.count, 1);
        assert_eq!(roster.entries.len(), 1);
        assert!(roster.entries[0].is_me);
    }

    #[test]
    fn without_a_ready_swarm_the_roster_is_disconnected() {
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .in_room(joined())
            .build();
        let mut app = app_with(state, None);
        app.update();
        let roster = app.world().resource::<discovery::RoomRoster>();
        assert_eq!(roster.own, discovery::OwnRoomStatus::Disconnected);
        assert_eq!(roster.entries, Vec::new());
    }

    #[test]
    fn without_a_room_the_roster_is_disconnected() {
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me")).build();
        let mut app = app_with(state, Some(p2p::ReadyChannel::default()));
        app.update();
        let roster = app.world().resource::<discovery::RoomRoster>();
        assert_eq!(roster.own, discovery::OwnRoomStatus::Disconnected);
    }
}
