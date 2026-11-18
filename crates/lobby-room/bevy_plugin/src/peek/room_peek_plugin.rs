use std::marker::PhantomData;

use bevy::prelude::*;

use crate::identity;
use crate::peek;
use crate::replica_sync;
use crate::spectator;
use crate::ui;

pub struct RoomPeekPlugin<S: replica_sync::traits::ReplicatedState> {
    pub game: identity::GameId,
    pub marker: PhantomData<S>,
}

impl<S: replica_sync::traits::ReplicatedState> RoomPeekPlugin<S> {
    #[must_use]
    pub const fn new(game: identity::GameId) -> Self {
        Self {
            game,
            marker: PhantomData,
        }
    }
}

#[rustfmt::skip]
impl<S: replica_sync::traits::ReplicatedState> bevy::prelude::Plugin for RoomPeekPlugin<S> {
    fn build(&self, app: &mut App) {
        app.insert_resource(peek::RoomPeekGame(self.game.clone()));
        app.add_message::<spectator::SpectateCommand>();
        app.add_plugins(spectator::SpectatorPlugin::<S>::new(self.game.clone()));
        app.add_systems(
            Update,
            (
                peek::bevy_systems::peek_selected_room,
                peek::bevy_systems::stream_streamed_room,
            )
                .before(replica_sync::ReplicaSyncSet::Drive),
        );
        app.add_systems(
            Update,
            (
                peek::bevy_systems::sync_preview_target::<S>,
                peek::bevy_systems::sync_stream_target::<S>,
                ui::discovery::bevy_systems::show_preview_image::<S>,
                ui::discovery::bevy_systems::show_stream_image::<S>,
            )
                .chain()
                .after(replica_sync::ReplicaSyncSet::Drive),
        );
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::time::Duration;

    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;

    use super::RoomPeekPlugin;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;
    use crate::peek;
    use crate::replica_sync;
    use crate::spectator;
    use crate::time;
    use crate::ui;

    #[derive(Default, Resource)]
    struct State;

    impl replica_sync::traits::ReplicatedState for State {
        type Op = u8;
        type Snapshot = u8;

        fn apply(&mut self, _meta: replica_sync::OpMeta, _op: &Self::Op) {}

        fn snapshot(&self) -> Self::Snapshot {
            0
        }

        fn merge(&mut self, _snapshot: Self::Snapshot) {}

        fn slice_content_hashes(&self) -> replica_sync::ReplicaSliceContentHashes {
            Vec::new()
        }

        fn slice_snapshot(&self, _index: replica_sync::ReplicaSliceIndex) -> Self::Snapshot {
            0
        }
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RoomPeekPlugin::<State>::new(identity::GameId::from("game")));
        assert!(app.world().get_resource::<peek::RoomPeekGame>().is_some());
        assert!(
            app.world()
                .get_resource::<Messages<spectator::SpectateCommand>>()
                .is_some()
        );
    }

    const fn preview_noop(_world: &mut World) {}

    fn spawn_mock_screen(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        commands.spawn((
            ChildOf(root),
            Node {
                width: Val::Percent(70.0),
                height: Val::Percent(50.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.20, 0.42, 0.62)),
        ));
        commands.spawn((
            ChildOf(root),
            Text::new("RoomPeekPlugin"),
            TextFont {
                font_size: FontSize::Px(24.0),
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    }

    fn preview_build(app: &mut App) {
        app.add_plugins(RoomPeekPlugin::<State> {
            game: identity::GameId::from("game"),
            marker: std::marker::PhantomData,
        });
        app.add_systems(Startup, spawn_mock_screen);
    }

    #[test]
    fn room_peek_plugin_ui_scene_preview() {
        lele_bevy_preview::scene::Scene {
            name: String::from("room_peek_plugin"),
            kind: lele_bevy_preview::scene::Kind::Plugin,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("lobby"),
                apply: preview_noop,
            }],
            timeline: None,
        }
        .run(&lele_bevy_preview::preview::Config::default())
        .expect("preview");
    }

    #[test]
    fn the_peek_plugin_shows_the_live_preview_and_restores_the_placeholder() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RoomPeekPlugin::<State>::new(identity::GameId::from("game")));
        let image = app
            .world_mut()
            .spawn((
                ui::discovery::RoomPreviewImage,
                ImageNode::default(),
                Visibility::Hidden,
                Node::default(),
            ))
            .id();
        let placeholder = app
            .world_mut()
            .spawn((
                ui::discovery::RoomPreviewPlaceholder,
                Visibility::Inherited,
                Node::default(),
            ))
            .id();
        let zone = identity::ZoneName::from("alpha");
        app.insert_resource(peek::RoomPreviewTarget {
            zone: zone.clone(),
            image: bevy::asset::Handle::default(),
        });
        let spectated = app
            .world_mut()
            .spawn(spectator::Spectated::<State> {
                slot: spectator::SpectateSlot::Preview,
                zone: zone.clone(),
                state: State,
                freshness: spectator::SpectateFreshness::Bootstrapping,
            })
            .id();
        let visibility = |app: &App, entity| {
            *app.world()
                .get::<Visibility>(entity)
                .expect("node visibility")
        };
        app.update();
        assert_eq!(visibility(&app, image), Visibility::Hidden);
        assert_eq!(visibility(&app, placeholder), Visibility::Inherited);
        for freshness in [
            spectator::SpectateFreshness::Live,
            spectator::SpectateFreshness::Repairing,
        ] {
            app.world_mut()
                .entity_mut(spectated)
                .insert(spectator::Spectated::<State> {
                    slot: spectator::SpectateSlot::Preview,
                    zone: zone.clone(),
                    state: State,
                    freshness,
                });
            app.update();
            assert_eq!(visibility(&app, image), Visibility::Visible);
            assert_eq!(visibility(&app, placeholder), Visibility::Hidden);
        }
        app.world_mut().despawn(spectated);
        app.update();
        assert_eq!(visibility(&app, image), Visibility::Hidden);
        assert_eq!(visibility(&app, placeholder), Visibility::Inherited);
    }

    #[test]
    fn the_peek_plugin_shows_the_live_stream_and_restores_the_placeholder() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RoomPeekPlugin::<State>::new(identity::GameId::from("game")));
        let image = app
            .world_mut()
            .spawn((
                ui::discovery::RoomStreamImage,
                ImageNode::default(),
                Visibility::Hidden,
                Node::default(),
            ))
            .id();
        let placeholder = app
            .world_mut()
            .spawn((
                ui::discovery::RoomStreamPlaceholder,
                Visibility::Inherited,
                Node::default(),
            ))
            .id();
        let zone = identity::ZoneName::from("alpha");
        app.insert_resource(peek::RoomStreamTarget {
            zone: zone.clone(),
            image: bevy::asset::Handle::default(),
        });
        let spectated = app
            .world_mut()
            .spawn(spectator::Spectated::<State> {
                slot: spectator::SpectateSlot::Stream,
                zone: zone.clone(),
                state: State,
                freshness: spectator::SpectateFreshness::Bootstrapping,
            })
            .id();
        let visibility = |app: &App, entity| {
            *app.world()
                .get::<Visibility>(entity)
                .expect("node visibility")
        };
        app.update();
        assert_eq!(visibility(&app, image), Visibility::Hidden);
        assert_eq!(visibility(&app, placeholder), Visibility::Inherited);
        for freshness in [
            spectator::SpectateFreshness::Live,
            spectator::SpectateFreshness::Repairing,
        ] {
            app.world_mut()
                .entity_mut(spectated)
                .insert(spectator::Spectated::<State> {
                    slot: spectator::SpectateSlot::Stream,
                    zone: zone.clone(),
                    state: State,
                    freshness,
                });
            app.update();
            assert_eq!(visibility(&app, image), Visibility::Visible);
            assert_eq!(visibility(&app, placeholder), Visibility::Hidden);
        }
        app.world_mut().despawn(spectated);
        app.update();
        assert_eq!(visibility(&app, image), Visibility::Hidden);
        assert_eq!(visibility(&app, placeholder), Visibility::Inherited);
    }
    fn game() -> identity::GameId {
        identity::GameId::from("game")
    }

    fn lobby_state(me: &str, rooms: &[(&str, u64)]) -> discovery::state_machine::DiscoveryState {
        let mut state = discovery::state_machine::DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from(me),
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

    fn peek_app_as(me: &str, rooms: &[(&str, u64)]) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            50,
        )));
        app.add_message::<p2p::Libp2pCommand>();
        app.insert_resource(ui::discovery::SelectedRoom::default());
        app.insert_resource(ui::discovery::StreamedRoom::default());
        app.insert_resource(lobby_state(me, rooms));
        app.init_resource::<Assets<Image>>();
        app.add_plugins(RoomPeekPlugin::<State>::new(game()));
        app
    }

    fn peek_app(rooms: &[(&str, u64)]) -> App {
        peek_app_as("me", rooms)
    }

    fn select(app: &mut App, room: Option<&str>) {
        **app
            .world_mut()
            .resource_mut::<ui::discovery::SelectedRoom>() = room.map(discovery::RoomName::from);
    }

    fn stream(app: &mut App, room: Option<&str>) {
        **app
            .world_mut()
            .resource_mut::<ui::discovery::StreamedRoom>() = room.map(discovery::RoomName::from);
    }

    fn spectate_commands(app: &mut App) -> Vec<spectator::SpectateCommand> {
        app.world_mut()
            .resource_mut::<Messages<spectator::SpectateCommand>>()
            .drain()
            .collect()
    }

    fn dials(app: &mut App) -> Vec<p2p::Libp2pCommand> {
        app.world_mut()
            .resource_mut::<Messages<p2p::Libp2pCommand>>()
            .drain()
            .collect()
    }

    fn start_for(
        slot: spectator::SpectateSlot,
        room: &str,
        created_at: u64,
    ) -> spectator::SpectateCommand {
        let room = discovery::RoomName::from(room);
        let prefix =
            p2p::HistoryKeyPrefix::new(&game(), &room, time::UnixTime::from_secs(created_at));
        spectator::SpectateCommand::Start {
            slot,
            zone: (*room).clone(),
            history_prefix: prefix,
        }
    }

    fn member_with_address(
        id: &str,
        membership: discovery::RoomMembership,
    ) -> discovery::RoomMember {
        discovery::RoomMember {
            peer: identity::Peer {
                id: identity::PeerId::from(id),
                addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/4001")],
            },
            display_name: None,
            updated_at: time::UnixTime::EPOCH,
            membership,
        }
    }

    fn add_member(app: &mut App, room: &str, member: &discovery::RoomMember) {
        app.world_mut()
            .resource_mut::<discovery::state_machine::DiscoveryState>()
            .lobby
            .get_mut(&discovery::RoomName::from(room))
            .expect("room in lobby")
            .members
            .insert(member.peer.id.clone(), member.clone());
    }

    fn rendezvous_room(app: &mut App) {
        for member in [
            member_with_address("member-a", discovery::RoomMembership::HistoryHolder),
            member_with_address("member-b", discovery::RoomMembership::Occupant),
            member_with_address("member-c", discovery::RoomMembership::HistoryHolder),
            member_with_address("member-d", discovery::RoomMembership::Occupant),
            member_with_address("member-e", discovery::RoomMembership::HistoryHolder),
        ] {
            add_member(app, "alpha", &member);
        }
    }

    fn dialed_member_ids(me: &str) -> Vec<String> {
        let mut app = peek_app_as(me, &[("alpha", 10)]);
        rendezvous_room(&mut app);
        select(&mut app, Some("alpha"));
        app.update();
        dials(&mut app)
            .into_iter()
            .filter_map(|command| match command {
                p2p::Libp2pCommand::Dial(peer) => Some(peer.id.as_str().to_string()),
                _ => None,
            })
            .collect()
    }

    fn spectated(slot: spectator::SpectateSlot, zone: &str) -> spectator::Spectated<State> {
        spectator::Spectated {
            slot,
            zone: identity::ZoneName::from(zone),
            state: State,
            freshness: spectator::SpectateFreshness::Live,
        }
    }

    fn despawn_spectated(app: &mut App) {
        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<spectator::Spectated<State>>>();
        let entities: Vec<Entity> = query.iter(app.world()).collect();
        for entity in entities {
            app.world_mut().despawn(entity);
        }
    }

    #[test]
    fn selecting_a_room_starts_its_preview() {
        let mut app = peek_app(&[("alpha", 10)]);
        select(&mut app, Some("alpha"));
        app.update();
        assert_eq!(
            spectate_commands(&mut app),
            vec![start_for(spectator::SpectateSlot::Preview, "alpha", 10)]
        );
    }

    #[test]
    fn starting_a_preview_dials_a_room_member_that_has_an_address() {
        let mut app = peek_app(&[("alpha", 10)]);
        let member = member_with_address("member", discovery::RoomMembership::Occupant);
        add_member(&mut app, "alpha", &member);
        select(&mut app, Some("alpha"));
        app.update();
        let _ = spectate_commands(&mut app);
        assert_eq!(dials(&mut app), vec![p2p::Libp2pCommand::Dial(member.peer)]);
    }

    #[test]
    fn selecting_without_a_p2p_channel_still_starts_the_preview() {
        let mut app = peek_app(&[("alpha", 10)]);
        app.world_mut()
            .remove_resource::<Messages<p2p::Libp2pCommand>>();
        select(&mut app, Some("alpha"));
        app.update();
        assert_eq!(
            spectate_commands(&mut app),
            vec![start_for(spectator::SpectateSlot::Preview, "alpha", 10)]
        );
    }

    #[test]
    fn the_same_room_selected_again_is_not_started_twice() {
        let mut app = peek_app(&[("alpha", 10)]);
        select(&mut app, Some("alpha"));
        app.update();
        let _ = spectate_commands(&mut app);
        app.update();
        assert_eq!(spectate_commands(&mut app), Vec::new());
    }

    #[test]
    fn a_cleared_selection_stops_only_after_the_linger() {
        let mut app = peek_app(&[("alpha", 10)]);
        select(&mut app, Some("alpha"));
        app.update();
        let _ = spectate_commands(&mut app);
        select(&mut app, None);
        app.update();
        assert_eq!(
            spectate_commands(&mut app),
            Vec::new(),
            "the stop is not immediate"
        );
        let mut stops = Vec::new();
        for _ in 0..40 {
            app.update();
            stops.extend(spectate_commands(&mut app));
            if !stops.is_empty() {
                break;
            }
        }
        assert_eq!(
            stops,
            vec![spectator::SpectateCommand::Stop {
                slot: spectator::SpectateSlot::Preview,
            }]
        );
    }

    #[test]
    fn re_hovering_within_the_linger_cancels_the_stop() {
        let mut app = peek_app(&[("alpha", 10)]);
        select(&mut app, Some("alpha"));
        app.update();
        let _ = spectate_commands(&mut app);
        select(&mut app, None);
        for _ in 0..5 {
            app.update();
        }
        select(&mut app, Some("alpha"));
        for _ in 0..30 {
            app.update();
        }
        assert_eq!(spectate_commands(&mut app), Vec::new());
    }

    #[test]
    fn a_different_room_starts_immediately_without_a_stop() {
        let mut app = peek_app(&[("alpha", 10), ("beta", 20)]);
        select(&mut app, Some("alpha"));
        app.update();
        let _ = spectate_commands(&mut app);
        select(&mut app, Some("beta"));
        app.update();
        assert_eq!(
            spectate_commands(&mut app),
            vec![start_for(spectator::SpectateSlot::Preview, "beta", 20)]
        );
    }

    #[test]
    fn a_room_missing_from_the_lobby_is_not_started() {
        let mut app = peek_app(&[("alpha", 10)]);
        select(&mut app, Some("missing"));
        app.update();
        assert_eq!(spectate_commands(&mut app), Vec::new());
    }

    #[test]
    fn the_streamed_room_gets_no_preview() {
        let mut app = peek_app(&[("alpha", 10)]);
        stream(&mut app, Some("alpha"));
        select(&mut app, Some("alpha"));
        app.update();
        let commands = spectate_commands(&mut app);
        assert!(
            !commands.contains(&start_for(spectator::SpectateSlot::Preview, "alpha", 10)),
            "the streamed room is not previewed: {commands:?}"
        );
    }

    #[test]
    fn hovering_the_streamed_room_stops_an_earlier_preview_after_the_linger() {
        let mut app = peek_app(&[("alpha", 10)]);
        select(&mut app, Some("alpha"));
        app.update();
        let _ = spectate_commands(&mut app);
        stream(&mut app, Some("alpha"));
        let mut stops = Vec::new();
        for _ in 0..40 {
            app.update();
            stops.extend(
                spectate_commands(&mut app)
                    .into_iter()
                    .filter(|command| matches!(command, spectator::SpectateCommand::Stop { .. })),
            );
            if !stops.is_empty() {
                break;
            }
        }
        assert_eq!(
            stops,
            vec![spectator::SpectateCommand::Stop {
                slot: spectator::SpectateSlot::Preview,
            }]
        );
    }

    #[test]
    fn hovering_another_room_while_streaming_previews_only_that_room() {
        let mut app = peek_app(&[("alpha", 10), ("beta", 20)]);
        stream(&mut app, Some("alpha"));
        app.update();
        let _ = spectate_commands(&mut app);
        select(&mut app, Some("beta"));
        app.update();
        assert_eq!(
            spectate_commands(&mut app),
            vec![start_for(spectator::SpectateSlot::Preview, "beta", 20)]
        );
    }

    #[test]
    fn streaming_a_room_starts_its_stream() {
        let mut app = peek_app(&[("alpha", 10)]);
        stream(&mut app, Some("alpha"));
        app.update();
        assert_eq!(
            spectate_commands(&mut app),
            vec![start_for(spectator::SpectateSlot::Stream, "alpha", 10)]
        );
    }

    #[test]
    fn starting_a_stream_dials_a_room_member_that_has_an_address() {
        let mut app = peek_app(&[("alpha", 10)]);
        let member = member_with_address("member", discovery::RoomMembership::Occupant);
        add_member(&mut app, "alpha", &member);
        stream(&mut app, Some("alpha"));
        app.update();
        let _ = spectate_commands(&mut app);
        assert_eq!(dials(&mut app), vec![p2p::Libp2pCommand::Dial(member.peer)]);
    }

    #[test]
    fn a_different_room_streams_immediately_without_a_stop() {
        let mut app = peek_app(&[("alpha", 10), ("beta", 20)]);
        stream(&mut app, Some("alpha"));
        app.update();
        let _ = spectate_commands(&mut app);
        stream(&mut app, Some("beta"));
        app.update();
        assert_eq!(
            spectate_commands(&mut app),
            vec![start_for(spectator::SpectateSlot::Stream, "beta", 20)]
        );
    }

    #[test]
    fn clearing_the_streamed_room_stops_immediately_without_a_linger() {
        let mut app = peek_app(&[("alpha", 10)]);
        stream(&mut app, Some("alpha"));
        app.update();
        let _ = spectate_commands(&mut app);
        stream(&mut app, None);
        app.update();
        assert_eq!(
            spectate_commands(&mut app),
            vec![spectator::SpectateCommand::Stop {
                slot: spectator::SpectateSlot::Stream,
            }]
        );
    }

    #[test]
    fn a_room_missing_from_the_lobby_is_not_streamed() {
        let mut app = peek_app(&[("alpha", 10)]);
        stream(&mut app, Some("missing"));
        app.update();
        assert_eq!(spectate_commands(&mut app), Vec::new());
    }

    #[test]
    fn the_preview_prefers_a_member_who_is_not_a_player() {
        let dialed = dialed_member_ids("spectator");
        assert_eq!(dialed.len(), 1, "one dial: {dialed:?}");
        assert!(
            matches!(dialed[0].as_str(), "member-a" | "member-c" | "member-e"),
            "dialed {}, expected a history holder",
            dialed[0]
        );
    }

    #[test]
    fn two_spectators_dial_different_members() {
        assert_ne!(
            dialed_member_ids("spectator-a"),
            dialed_member_ids("spectator-b")
        );
    }

    #[test]
    fn the_same_spectator_always_dials_the_same_member() {
        assert_eq!(
            dialed_member_ids("spectator-a"),
            dialed_member_ids("spectator-a")
        );
    }

    #[test]
    fn the_preview_never_dials_itself() {
        let dialed = dialed_member_ids("member-c");
        assert_ne!(dialed, vec![String::from("member-c")]);
        let mut solo = peek_app_as("solo", &[("alpha", 10)]);
        add_member(
            &mut solo,
            "alpha",
            &member_with_address("solo", discovery::RoomMembership::HistoryHolder),
        );
        select(&mut solo, Some("alpha"));
        solo.update();
        assert_eq!(dials(&mut solo), Vec::new(), "there is nobody else to dial");
    }

    #[test]
    fn a_live_spectated_zone_creates_its_target() {
        let mut app = peek_app(&[]);
        app.world_mut()
            .spawn(spectated(spectator::SpectateSlot::Preview, "alpha"));
        app.world_mut()
            .spawn(spectated(spectator::SpectateSlot::Stream, "alpha"));
        app.update();
        assert_eq!(
            app.world().resource::<peek::RoomPreviewTarget>().zone,
            identity::ZoneName::from("alpha")
        );
        assert_eq!(
            app.world().resource::<peek::RoomStreamTarget>().zone,
            identity::ZoneName::from("alpha")
        );
        assert_eq!(app.world().resource::<Assets<Image>>().len(), 2);
    }

    #[test]
    fn leaving_a_spectated_zone_removes_its_target_and_image() {
        let mut app = peek_app(&[]);
        app.world_mut()
            .spawn(spectated(spectator::SpectateSlot::Preview, "alpha"));
        app.world_mut()
            .spawn(spectated(spectator::SpectateSlot::Stream, "alpha"));
        app.update();
        despawn_spectated(&mut app);
        app.update();
        assert!(
            app.world()
                .get_resource::<peek::RoomPreviewTarget>()
                .is_none()
        );
        assert!(
            app.world()
                .get_resource::<peek::RoomStreamTarget>()
                .is_none()
        );
        assert_eq!(app.world().resource::<Assets<Image>>().len(), 0);
    }

    #[test]
    fn switching_rooms_keeps_the_target_image_and_updates_the_zone() {
        let mut app = peek_app(&[]);
        app.world_mut()
            .spawn(spectated(spectator::SpectateSlot::Preview, "alpha"));
        app.world_mut()
            .spawn(spectated(spectator::SpectateSlot::Stream, "alpha"));
        app.update();
        let preview_image = app
            .world()
            .resource::<peek::RoomPreviewTarget>()
            .image
            .clone();
        let stream_image = app
            .world()
            .resource::<peek::RoomStreamTarget>()
            .image
            .clone();
        despawn_spectated(&mut app);
        app.world_mut()
            .spawn(spectated(spectator::SpectateSlot::Preview, "beta"));
        app.world_mut()
            .spawn(spectated(spectator::SpectateSlot::Stream, "beta"));
        app.update();
        let preview = app.world().resource::<peek::RoomPreviewTarget>();
        assert_eq!(preview.zone, identity::ZoneName::from("beta"));
        assert_eq!(preview.image, preview_image);
        let stream_target = app.world().resource::<peek::RoomStreamTarget>();
        assert_eq!(stream_target.zone, identity::ZoneName::from("beta"));
        assert_eq!(stream_target.image, stream_image);
        assert_eq!(app.world().resource::<Assets<Image>>().len(), 2);
    }
}
