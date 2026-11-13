use bevy::prelude::*;
use discovery::state_machine::DiscoveryState;
use freenet_libp2p_bevy_plugin::lobby_room::discovery;
use freenet_libp2p_bevy_plugin::time;

use crate::starboard;

pub fn refresh_room_brushes(
    time: Res<Time>,
    status: Option<Res<DiscoveryState>>,
    mut timer: ResMut<starboard::room::BrushRefreshTimer>,
    mut brushes: ResMut<starboard::room::RoomBrushes>,
    mut own: ResMut<starboard::input::OwnBrush>,
) {
    let time = time.into_inner();
    if !timer.0.tick(time.delta()).is_finished() {
        return;
    }
    let Some(status) = status else {
        return;
    };
    let status = status.into_inner();
    let Some(joined) = status.room.as_ref() else {
        return;
    };
    let now = time::UnixTime::now();
    let next =
        starboard::identity::room_brushes(status.occupants().as_slice(), joined.created_at, now);
    if next != brushes.0 {
        brushes.0 = next;
    }
    if let Some(brush) = brushes.0.get(&status.me.id) {
        own.shape = brush.shape;
        own.color = brush.color;
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;
    use discovery::state_machine::DiscoveryState;
    use freenet_libp2p_bevy_plugin::identity;
    use freenet_libp2p_bevy_plugin::lobby_room::discovery;
    use freenet_libp2p_bevy_plugin::time;

    use super::refresh_room_brushes;
    use crate::starboard;

    fn member(peer: &str) -> discovery::RoomMember {
        discovery::RoomMember {
            peer: identity::Peer {
                id: identity::PeerId::from(peer),
                addrs: Vec::new(),
            },
            updated_at: time::UnixTime::EPOCH,
            display_name: None,
            membership: discovery::RoomMembership::Occupant,
        }
    }

    fn ready_room(me: &str, member_names: &[&str]) -> DiscoveryState {
        let mut room_members = discovery::Members::new();
        for name in member_names {
            let peer = identity::PeerId::from(*name);
            room_members.insert(peer.clone(), member(name));
        }
        let mut state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from(me),
            addrs: Vec::new(),
        });
        state.room = Some(discovery::JoinedRoom {
            name: identity::RoomName::from("alpha"),
            created_at: time::UnixTime::from_secs(1),
            phase: discovery::JoinPhase::Ready,
            members: room_members,
            how_entered: discovery::HowRoomWasEntered::Created,
        });
        state
    }

    fn app_with(status: DiscoveryState) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(status);
        app.insert_resource(starboard::room::BrushRefreshTimer(Timer::from_seconds(
            0.0,
            TimerMode::Repeating,
        )));
        app.init_resource::<starboard::room::RoomBrushes>();
        app.init_resource::<starboard::input::OwnBrush>();
        app.add_systems(Update, refresh_room_brushes);
        app
    }

    fn run_one_refresh(app: &mut App) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs(1));
        app.update();
    }

    #[test]
    fn test_usage() {
        let mut app = app_with(ready_room("me", &["peer-a", "peer-b"]));
        run_one_refresh(&mut app);

        let brushes = app.world().resource::<starboard::room::RoomBrushes>();
        assert_eq!(brushes.len(), 3);
        let me = identity::PeerId::from("me");
        let mine = brushes
            .get(&me)
            .copied()
            .expect("own id present in room brushes");
        let own = app.world().resource::<starboard::input::OwnBrush>();
        assert_eq!(own.shape, mine.shape);
        assert_eq!(own.color, mine.color);
    }

    #[test]
    fn test_usage_1() {
        let peer_a = identity::PeerId::from("peer-a");
        let peer_b = identity::PeerId::from("peer-b");
        let mut app_a = app_with(ready_room(peer_a.as_str(), &[peer_b.as_str()]));
        let mut app_b = app_with(ready_room(peer_b.as_str(), &[peer_a.as_str()]));
        run_one_refresh(&mut app_a);
        run_one_refresh(&mut app_b);

        let brushes_a = app_a
            .world()
            .resource::<starboard::room::RoomBrushes>()
            .0
            .clone();
        let brushes_b = app_b
            .world()
            .resource::<starboard::room::RoomBrushes>()
            .0
            .clone();
        assert_eq!(brushes_a.len(), 2);
        assert_eq!(brushes_a, brushes_b);
        assert_ne!(
            brushes_a.get(&peer_a),
            brushes_a.get(&peer_b),
            "the two peers paint with distinct brushes"
        );
    }
}
