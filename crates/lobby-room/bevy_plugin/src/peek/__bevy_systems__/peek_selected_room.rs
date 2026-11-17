use bevy::prelude::*;

use crate::discovery;
use crate::p2p;
use crate::peek;
use crate::spectator;
use crate::ui;

use ui::discovery::SelectedRoom;

pub fn peek_selected_room(
    selection: Option<Res<SelectedRoom>>,
    discovery: Option<Res<discovery::state_machine::DiscoveryState>>,
    game: Option<Res<peek::RoomPeekGame>>,
    time: Res<Time>,
    mut started_room: Local<Option<discovery::RoomName>>,
    mut cleared_for_secs: Local<Option<f32>>,
    mut spectate: MessageWriter<spectator::SpectateCommand>,
    mut network: MessageWriter<p2p::Libp2pCommand>,
) {
    let Some(selection) = selection else {
        return;
    };
    let Some(room) = selection.room.as_ref() else {
        if started_room.is_none() {
            return;
        }
        let elapsed = cleared_for_secs.unwrap_or(0.0) + time.into_inner().delta_secs();
        if elapsed < peek::PREVIEW_STOP_LINGER_SECS {
            *cleared_for_secs = Some(elapsed);
            return;
        }
        *started_room = None;
        *cleared_for_secs = None;
        spectate.write(spectator::SpectateCommand::Stop);
        return;
    };
    *cleared_for_secs = None;
    if started_room.as_ref() == Some(room) {
        return;
    }
    let (Some(state), Some(game)) = (discovery.map(Res::into_inner), game.map(Res::into_inner))
    else {
        return;
    };
    let Some(room_record) = state.lobby.get(room) else {
        return;
    };
    let history_prefix = p2p::HistoryKeyPrefix::new(game, room, room_record.created_at);
    *started_room = Some(room.clone());
    if let Some(member) = room_record
        .members
        .values()
        .find(|member| !member.peer.addrs.is_empty())
    {
        network.write(p2p::Libp2pCommand::Dial(member.peer.clone()));
    }
    spectate.write(spectator::SpectateCommand::Start {
        zone: (**room).clone(),
        history_prefix,
    });
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::time::Duration;

    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;

    use super::peek_selected_room;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;
    use crate::peek;
    use crate::spectator;
    use crate::time;
    use crate::ui;

    fn game() -> identity::GameId {
        identity::GameId::from("game")
    }

    fn lobby_state(rooms: &[(&str, u64)]) -> discovery::state_machine::DiscoveryState {
        let mut state = discovery::state_machine::DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        for (name, created_at) in rooms {
            state.lobby.insert(
                discovery::RoomName::from(*name),
                discovery::Room {
                    room_size: discovery::RoomSize::DEFAULT,
                    created_at: time::UnixTime::from_secs(*created_at),
                    members: BTreeMap::new(),
                },
            );
        }
        state
    }

    fn app_with(rooms: &[(&str, u64)]) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            50,
        )));
        app.add_message::<spectator::SpectateCommand>();
        app.add_message::<p2p::Libp2pCommand>();
        app.insert_resource(ui::discovery::SelectedRoom::default());
        app.insert_resource(lobby_state(rooms));
        app.insert_resource(peek::RoomPeekGame(game()));
        app.add_systems(Update, peek_selected_room);
        app
    }

    fn select(app: &mut App, room: Option<&str>) {
        app.world_mut()
            .resource_mut::<ui::discovery::SelectedRoom>()
            .room = room.map(discovery::RoomName::from);
    }

    fn sent(app: &mut App) -> Vec<spectator::SpectateCommand> {
        app.world_mut()
            .resource_mut::<Messages<spectator::SpectateCommand>>()
            .drain()
            .collect()
    }

    fn drained_network(app: &mut App) -> Vec<p2p::Libp2pCommand> {
        app.world_mut()
            .resource_mut::<Messages<p2p::Libp2pCommand>>()
            .drain()
            .collect()
    }

    fn start_for(room: &str, created_at: u64) -> spectator::SpectateCommand {
        let room = discovery::RoomName::from(room);
        let prefix =
            p2p::HistoryKeyPrefix::new(&game(), &room, time::UnixTime::from_secs(created_at));
        spectator::SpectateCommand::Start {
            zone: (*room).clone(),
            history_prefix: prefix,
        }
    }

    #[test]
    fn test_usage() {
        let mut app = app_with(&[("alpha", 10)]);
        select(&mut app, Some("alpha"));
        app.update();
        assert_eq!(sent(&mut app), vec![start_for("alpha", 10)]);
    }

    #[test]
    fn starting_a_preview_dials_a_room_member_that_has_an_address() {
        let mut app = app_with(&[("alpha", 10)]);
        let member = discovery::RoomMember {
            peer: identity::Peer {
                id: identity::PeerId::from("member"),
                addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/4001")],
            },
            display_name: None,
            updated_at: time::UnixTime::EPOCH,
            membership: discovery::RoomMembership::Occupant,
        };
        app.world_mut()
            .resource_mut::<discovery::state_machine::DiscoveryState>()
            .lobby
            .get_mut(&discovery::RoomName::from("alpha"))
            .expect("alpha room")
            .members
            .insert(member.peer.id.clone(), member.clone());
        select(&mut app, Some("alpha"));
        app.update();
        let _ = sent(&mut app);
        assert_eq!(
            drained_network(&mut app),
            vec![p2p::Libp2pCommand::Dial(member.peer)]
        );
    }

    #[test]
    fn the_same_room_selected_again_is_not_started_twice() {
        let mut app = app_with(&[("alpha", 10)]);
        select(&mut app, Some("alpha"));
        app.update();
        let _ = sent(&mut app);
        app.update();
        assert_eq!(sent(&mut app), Vec::new());
    }

    #[test]
    fn a_cleared_selection_stops_only_after_the_linger() {
        let mut app = app_with(&[("alpha", 10)]);
        select(&mut app, Some("alpha"));
        app.update();
        let _ = sent(&mut app);
        select(&mut app, None);
        for _ in 0..19 {
            app.update();
        }
        assert_eq!(sent(&mut app), Vec::new(), "still inside the linger");
        app.update();
        assert_eq!(sent(&mut app), vec![spectator::SpectateCommand::Stop]);
    }

    #[test]
    fn re_hovering_within_the_linger_cancels_the_stop() {
        let mut app = app_with(&[("alpha", 10)]);
        select(&mut app, Some("alpha"));
        app.update();
        let _ = sent(&mut app);
        select(&mut app, None);
        for _ in 0..5 {
            app.update();
        }
        select(&mut app, Some("alpha"));
        for _ in 0..30 {
            app.update();
        }
        assert_eq!(sent(&mut app), Vec::new());
    }

    #[test]
    fn a_different_room_starts_immediately_without_a_stop() {
        let mut app = app_with(&[("alpha", 10), ("beta", 20)]);
        select(&mut app, Some("alpha"));
        app.update();
        let _ = sent(&mut app);
        select(&mut app, Some("beta"));
        app.update();
        assert_eq!(sent(&mut app), vec![start_for("beta", 20)]);
    }

    #[test]
    fn a_room_missing_from_the_lobby_is_not_started() {
        let mut app = app_with(&[("alpha", 10)]);
        select(&mut app, Some("missing"));
        app.update();
        assert_eq!(sent(&mut app), Vec::new());
    }
}
