use bevy::prelude::*;

use crate::discovery;
use discovery::freenet::basic::constants::MIN_RING_PEERS;
use discovery::ui::Screen;

pub fn pick_screen(
    snapshot: Res<discovery::Snapshot>,
    screen: Res<State<Screen>>,
    mut next: ResMut<NextState<Screen>>,
) {
    let snapshot = snapshot.into_inner();
    let current = *screen.into_inner().get();
    let ring_ready = discovery::freenet::is_ring_ready(&snapshot.freenet_peers, MIN_RING_PEERS);
    let phase = snapshot.room.as_ref().map(|room| room.phase);
    let wanted = match (phase, current) {
        (Some(discovery::RoomPhase::Ready), _) => Screen::InRoom,
        (Some(discovery::RoomPhase::Joining), _) => Screen::JoiningRoom,
        (None, Screen::Connecting) if !ring_ready => Screen::Connecting,
        (None, _) => Screen::Lobby,
    };
    if wanted != current {
        tracing::info!(target: "discovery", "ui screen {current:?} -> {wanted:?}");
        next.set(wanted);
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::state::app::StatesPlugin;

    use super::pick_screen;
    use crate::{discovery, identity};

    fn ready_peers() -> Vec<identity::FreenetPeer> {
        (1..=3_u16)
            .map(|port| identity::FreenetPeer {
                id: identity::FreenetPeerId::from(format!("peer-{port}").as_str()),
                addr: std::net::SocketAddr::from(([127, 0, 0, 1], port)),
            })
            .collect()
    }

    fn screen_after(snapshot: discovery::Snapshot) -> discovery::ui::Screen {
        let mut app = App::new();
        app.add_plugins(StatesPlugin);
        app.init_state::<discovery::ui::Screen>();
        app.insert_resource(snapshot);
        app.add_systems(Update, pick_screen);
        app.update();
        app.update();
        *app.world().resource::<State<discovery::ui::Screen>>().get()
    }

    #[test]
    fn test_usage() {
        assert_eq!(
            screen_after(discovery::Snapshot::default()),
            discovery::ui::Screen::Connecting
        );
        let ready = discovery::Snapshot {
            freenet_peers: ready_peers(),
            ..Default::default()
        };
        assert_eq!(screen_after(ready.clone()), discovery::ui::Screen::Lobby);
        let joining = discovery::Snapshot {
            room: Some(discovery::JoinedRoom {
                name: identity::RoomName::from("alpha"),
                created_at: discovery::UnixTime::from_secs(0),
                phase: discovery::RoomPhase::Joining,
                members: discovery::Members::new(),
            }),
            ..ready.clone()
        };
        assert_eq!(screen_after(joining), discovery::ui::Screen::JoiningRoom);
        let joined = discovery::Snapshot {
            room: Some(discovery::JoinedRoom {
                name: identity::RoomName::from("alpha"),
                created_at: discovery::UnixTime::from_secs(0),
                phase: discovery::RoomPhase::Ready,
                members: discovery::Members::new(),
            }),
            ..ready
        };
        assert_eq!(screen_after(joined), discovery::ui::Screen::InRoom);
    }
}
