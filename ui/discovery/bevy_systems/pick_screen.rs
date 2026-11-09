use bevy::prelude::*;

use crate::discovery;
use crate::ui;
use discovery::state_machine::DiscoveryState;
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
            (Some(discovery::JoinPhase::Ready), _) => Screen::InRoom,
            (Some(discovery::JoinPhase::Joining { .. }), _) => Screen::JoiningRoom,
            (None, Screen::Connecting) if !is_lobby_fetched => Screen::Connecting,
            (None, _) => Screen::Lobby,
        }
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
    use crate::testing;
    use crate::time;
    use crate::ui;
    use crate::{discovery, identity};
    use discovery::state_machine::DiscoveryState;

    fn ready_peers() -> Vec<identity::FreenetPeer> {
        (1..=3_u16)
            .map(|port| identity::FreenetPeer {
                id: identity::FreenetPeerId::from(format!("peer-{port}").as_str()),
                addr: std::net::SocketAddr::from(([127, 0, 0, 1], port)),
            })
            .collect()
    }

    fn screen_after(state: Option<DiscoveryState>) -> ui::discovery::Screen {
        let mut app = App::new();
        app.add_plugins(StatesPlugin);
        app.init_state::<ui::discovery::Screen>();
        app.init_resource::<ui::discovery::JoinError>();
        if let Some(state) = state {
            app.insert_resource(state);
        }
        app.add_systems(Update, pick_screen);
        app.update();
        app.update();
        *app.world().resource::<State<ui::discovery::Screen>>().get()
    }

    fn empty_state() -> DiscoveryState {
        testing::DiscoveryStateBuilder::new(identity::PeerId::from("me")).build()
    }

    fn ready_state() -> DiscoveryState {
        testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_freenet_peers(ready_peers())
            .lobby_fetched()
            .build()
    }

    fn state_in(room: &str, phase: discovery::JoinPhase) -> DiscoveryState {
        testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_freenet_peers(ready_peers())
            .lobby_fetched()
            .in_room(discovery::JoinedRoom {
                name: identity::RoomName::from(room),
                created_at: time::UnixTime::EPOCH,
                phase,
                members: discovery::Members::new(),
                how_entered: discovery::HowRoomWasEntered::Created,
            })
            .build()
    }

    #[test]
    fn test_usage() {
        assert_eq!(screen_after(None), ui::discovery::Screen::Connecting);
        assert_eq!(
            screen_after(Some(empty_state())),
            ui::discovery::Screen::Connecting
        );
        assert_eq!(
            screen_after(Some(ready_state())),
            ui::discovery::Screen::Lobby
        );
        let joining = state_in(
            "alpha",
            discovery::JoinPhase::Joining {
                since: time::UnixTime::EPOCH,
            },
        );
        assert_eq!(
            screen_after(Some(joining)),
            ui::discovery::Screen::JoiningRoom
        );
        let joined = state_in("alpha", discovery::JoinPhase::Ready);
        assert_eq!(screen_after(Some(joined)), ui::discovery::Screen::InRoom);
    }
}
