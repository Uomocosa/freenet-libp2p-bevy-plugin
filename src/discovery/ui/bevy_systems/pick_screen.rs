use bevy::prelude::*;

use crate::discovery;
use discovery::ui::{JoinError, Screen};

pub fn pick_screen(
    status: Res<discovery::Status>,
    join_error: Option<Res<JoinError>>,
    screen: Res<State<Screen>>,
    mut next: ResMut<NextState<Screen>>,
) {
    let status = status.into_inner();
    let current = *screen.into_inner().get();
    let has_join_error = join_error.is_some_and(|err| err.is_some());
    let phase = status.room.as_ref().map(|room| room.phase);
    let wanted = if has_join_error {
        Screen::JoinFailed
    } else {
        match (phase, current) {
            (Some(discovery::JoinPhase::Ready), _) => Screen::InRoom,
            (Some(discovery::JoinPhase::Joining { .. }), _) => Screen::JoiningRoom,
            (None, Screen::Connecting) if !status.is_lobby_attached => Screen::Connecting,
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
    use crate::{discovery, identity};

    fn ready_peers() -> Vec<identity::FreenetPeer> {
        (1..=3_u16)
            .map(|port| identity::FreenetPeer {
                id: identity::FreenetPeerId::from(format!("peer-{port}").as_str()),
                addr: std::net::SocketAddr::from(([127, 0, 0, 1], port)),
            })
            .collect()
    }

    fn screen_after(status: discovery::Status) -> discovery::ui::Screen {
        let mut app = App::new();
        app.add_plugins(StatesPlugin);
        app.init_state::<discovery::ui::Screen>();
        app.init_resource::<discovery::ui::JoinError>();
        app.insert_resource(status);
        app.add_systems(Update, pick_screen);
        app.update();
        app.update();
        *app.world().resource::<State<discovery::ui::Screen>>().get()
    }

    #[test]
    fn test_usage() {
        assert_eq!(
            screen_after(discovery::Status::default()),
            discovery::ui::Screen::Connecting
        );
        let ready = discovery::Status {
            freenet_peers: ready_peers(),
            is_lobby_attached: true,
            ..Default::default()
        };
        assert_eq!(screen_after(ready.clone()), discovery::ui::Screen::Lobby);
        let joining = discovery::Status {
            room: Some(discovery::JoinedRoom {
                name: identity::RoomName::from("alpha"),
                created_at: discovery::UnixTime::from(0),
                phase: discovery::JoinPhase::Joining {
                    since: discovery::UnixTime::from(0),
                },
                members: discovery::Members::new(),
            }),
            ..ready.clone()
        };
        assert_eq!(screen_after(joining), discovery::ui::Screen::JoiningRoom);
        let joined = discovery::Status {
            room: Some(discovery::JoinedRoom {
                name: identity::RoomName::from("alpha"),
                created_at: discovery::UnixTime::from(0),
                phase: discovery::JoinPhase::Ready,
                members: discovery::Members::new(),
            }),
            ..ready
        };
        assert_eq!(screen_after(joined), discovery::ui::Screen::InRoom);
    }
}
