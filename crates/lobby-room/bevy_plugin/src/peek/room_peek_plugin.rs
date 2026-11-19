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
    use bevy::prelude::*;

    use super::RoomPeekPlugin;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;
    use crate::peek;
    use crate::spectator;
    use crate::testing;
    use crate::ui;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RoomPeekPlugin::<testing::FakeReplica<u8, u8>>::new(
            testing::peek_game(),
        ));
        assert!(app.world().get_resource::<peek::RoomPeekGame>().is_some());
        assert!(
            app.world()
                .get_resource::<Messages<spectator::SpectateCommand>>()
                .is_some()
        );
    }

    #[test]
    fn room_peek_plugin_ui_scene_preview() {
        lele_bevy_preview::scene::Scene {
            name: String::from("room_peek_plugin"),
            kind: lele_bevy_preview::scene::Kind::Plugin,
            build: |app: &mut App| {
                app.add_plugins(RoomPeekPlugin::<testing::FakeReplica<u8, u8>>::new(
                    testing::peek_game(),
                ));
                app.add_systems(Startup, |mut commands: Commands| {
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
                });
            },
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("lobby"),
                apply: |_world: &mut World| {},
            }],
            timeline: None,
        }
        .run(&lele_bevy_preview::preview::Config::default())
        .expect("preview");
    }

    #[test]
    fn the_peek_plugin_shows_the_live_preview_and_restores_the_placeholder() {
        let mut app = testing::peek_app("me", &[]);
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
        app.insert_resource(peek::RoomPreviewTarget {
            zone: identity::ZoneName::from("alpha"),
            image: bevy::asset::Handle::default(),
        });
        let spectated = app
            .world_mut()
            .spawn(testing::spectated_zone(
                spectator::SpectateSlot::Preview,
                "alpha",
                spectator::SpectateFreshness::Bootstrapping,
            ))
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
                .insert(testing::spectated_zone(
                    spectator::SpectateSlot::Preview,
                    "alpha",
                    freshness,
                ));
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
        let mut app = testing::peek_app("me", &[]);
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
        app.insert_resource(peek::RoomStreamTarget {
            zone: identity::ZoneName::from("alpha"),
            image: bevy::asset::Handle::default(),
        });
        let spectated = app
            .world_mut()
            .spawn(testing::spectated_zone(
                spectator::SpectateSlot::Stream,
                "alpha",
                spectator::SpectateFreshness::Bootstrapping,
            ))
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
                .insert(testing::spectated_zone(
                    spectator::SpectateSlot::Stream,
                    "alpha",
                    freshness,
                ));
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
    fn selecting_a_room_starts_its_preview() {
        let mut app = testing::peek_app("me", &[("alpha", 10)]);
        testing::select_room(&mut app, Some("alpha"));
        app.update();
        assert_eq!(
            testing::drained_messages::<spectator::SpectateCommand>(&mut app),
            vec![testing::spectate_start(
                spectator::SpectateSlot::Preview,
                "alpha",
                10
            )]
        );
    }

    #[test]
    fn starting_a_preview_dials_a_room_member_that_has_an_address() {
        let mut app = testing::peek_app("me", &[("alpha", 10)]);
        let member = testing::room_member(
            "member",
            None,
            discovery::RoomMembership::Occupant,
            &["/ip4/127.0.0.1/tcp/4001"],
        );
        testing::add_member(&mut app, "alpha", &member);
        testing::select_room(&mut app, Some("alpha"));
        app.update();
        let _ = testing::drained_messages::<spectator::SpectateCommand>(&mut app);
        assert_eq!(
            testing::drained_messages::<p2p::Libp2pCommand>(&mut app),
            vec![p2p::Libp2pCommand::Dial(member.peer)]
        );
    }

    #[test]
    fn selecting_without_a_p2p_channel_still_starts_the_preview() {
        let mut app = testing::peek_app("me", &[("alpha", 10)]);
        app.world_mut()
            .remove_resource::<Messages<p2p::Libp2pCommand>>();
        testing::select_room(&mut app, Some("alpha"));
        app.update();
        assert_eq!(
            testing::drained_messages::<spectator::SpectateCommand>(&mut app),
            vec![testing::spectate_start(
                spectator::SpectateSlot::Preview,
                "alpha",
                10
            )]
        );
    }

    #[test]
    fn the_same_room_selected_again_is_not_started_twice() {
        let mut app = testing::peek_app("me", &[("alpha", 10)]);
        testing::select_room(&mut app, Some("alpha"));
        app.update();
        let _ = testing::drained_messages::<spectator::SpectateCommand>(&mut app);
        app.update();
        assert_eq!(
            testing::drained_messages::<spectator::SpectateCommand>(&mut app),
            Vec::new()
        );
    }

    #[test]
    fn a_cleared_selection_stops_only_after_the_linger() {
        let mut app = testing::peek_app("me", &[("alpha", 10)]);
        testing::select_room(&mut app, Some("alpha"));
        app.update();
        let _ = testing::drained_messages::<spectator::SpectateCommand>(&mut app);
        testing::select_room(&mut app, None);
        app.update();
        assert_eq!(
            testing::drained_messages::<spectator::SpectateCommand>(&mut app),
            Vec::new(),
            "the stop is not immediate"
        );
        let mut stops = Vec::new();
        for _ in 0..40 {
            app.update();
            stops.extend(testing::drained_messages::<spectator::SpectateCommand>(
                &mut app,
            ));
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
        let mut app = testing::peek_app("me", &[("alpha", 10)]);
        testing::select_room(&mut app, Some("alpha"));
        app.update();
        let _ = testing::drained_messages::<spectator::SpectateCommand>(&mut app);
        testing::select_room(&mut app, None);
        for _ in 0..5 {
            app.update();
        }
        testing::select_room(&mut app, Some("alpha"));
        for _ in 0..30 {
            app.update();
        }
        assert_eq!(
            testing::drained_messages::<spectator::SpectateCommand>(&mut app),
            Vec::new()
        );
    }

    #[test]
    fn a_different_room_starts_immediately_without_a_stop() {
        let mut app = testing::peek_app("me", &[("alpha", 10), ("beta", 20)]);
        testing::select_room(&mut app, Some("alpha"));
        app.update();
        let _ = testing::drained_messages::<spectator::SpectateCommand>(&mut app);
        testing::select_room(&mut app, Some("beta"));
        app.update();
        assert_eq!(
            testing::drained_messages::<spectator::SpectateCommand>(&mut app),
            vec![testing::spectate_start(
                spectator::SpectateSlot::Preview,
                "beta",
                20
            )]
        );
    }

    #[test]
    fn a_room_missing_from_the_lobby_is_not_started() {
        let mut app = testing::peek_app("me", &[("alpha", 10)]);
        testing::select_room(&mut app, Some("missing"));
        app.update();
        assert_eq!(
            testing::drained_messages::<spectator::SpectateCommand>(&mut app),
            Vec::new()
        );
    }

    #[test]
    fn the_streamed_room_gets_no_preview() {
        let mut app = testing::peek_app("me", &[("alpha", 10)]);
        testing::stream_room(&mut app, Some("alpha"));
        testing::select_room(&mut app, Some("alpha"));
        app.update();
        let commands = testing::drained_messages::<spectator::SpectateCommand>(&mut app);
        assert!(
            !commands.contains(&testing::spectate_start(
                spectator::SpectateSlot::Preview,
                "alpha",
                10
            )),
            "the streamed room is not previewed: {commands:?}"
        );
    }

    #[test]
    fn hovering_the_streamed_room_stops_an_earlier_preview_after_the_linger() {
        let mut app = testing::peek_app("me", &[("alpha", 10)]);
        testing::select_room(&mut app, Some("alpha"));
        app.update();
        let _ = testing::drained_messages::<spectator::SpectateCommand>(&mut app);
        testing::stream_room(&mut app, Some("alpha"));
        let mut stops = Vec::new();
        for _ in 0..40 {
            app.update();
            stops.extend(
                testing::drained_messages::<spectator::SpectateCommand>(&mut app)
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
        let mut app = testing::peek_app("me", &[("alpha", 10), ("beta", 20)]);
        testing::stream_room(&mut app, Some("alpha"));
        app.update();
        let _ = testing::drained_messages::<spectator::SpectateCommand>(&mut app);
        testing::select_room(&mut app, Some("beta"));
        app.update();
        assert_eq!(
            testing::drained_messages::<spectator::SpectateCommand>(&mut app),
            vec![testing::spectate_start(
                spectator::SpectateSlot::Preview,
                "beta",
                20
            )]
        );
    }

    #[test]
    fn streaming_a_room_starts_its_stream() {
        let mut app = testing::peek_app("me", &[("alpha", 10)]);
        testing::stream_room(&mut app, Some("alpha"));
        app.update();
        assert_eq!(
            testing::drained_messages::<spectator::SpectateCommand>(&mut app),
            vec![testing::spectate_start(
                spectator::SpectateSlot::Stream,
                "alpha",
                10
            )]
        );
    }

    #[test]
    fn starting_a_stream_dials_a_room_member_that_has_an_address() {
        let mut app = testing::peek_app("me", &[("alpha", 10)]);
        let member = testing::room_member(
            "member",
            None,
            discovery::RoomMembership::Occupant,
            &["/ip4/127.0.0.1/tcp/4001"],
        );
        testing::add_member(&mut app, "alpha", &member);
        testing::stream_room(&mut app, Some("alpha"));
        app.update();
        let _ = testing::drained_messages::<spectator::SpectateCommand>(&mut app);
        assert_eq!(
            testing::drained_messages::<p2p::Libp2pCommand>(&mut app),
            vec![p2p::Libp2pCommand::Dial(member.peer)]
        );
    }

    #[test]
    fn a_different_room_streams_immediately_without_a_stop() {
        let mut app = testing::peek_app("me", &[("alpha", 10), ("beta", 20)]);
        testing::stream_room(&mut app, Some("alpha"));
        app.update();
        let _ = testing::drained_messages::<spectator::SpectateCommand>(&mut app);
        testing::stream_room(&mut app, Some("beta"));
        app.update();
        assert_eq!(
            testing::drained_messages::<spectator::SpectateCommand>(&mut app),
            vec![testing::spectate_start(
                spectator::SpectateSlot::Stream,
                "beta",
                20
            )]
        );
    }

    #[test]
    fn clearing_the_streamed_room_stops_immediately_without_a_linger() {
        let mut app = testing::peek_app("me", &[("alpha", 10)]);
        testing::stream_room(&mut app, Some("alpha"));
        app.update();
        let _ = testing::drained_messages::<spectator::SpectateCommand>(&mut app);
        testing::stream_room(&mut app, None);
        app.update();
        assert_eq!(
            testing::drained_messages::<spectator::SpectateCommand>(&mut app),
            vec![spectator::SpectateCommand::Stop {
                slot: spectator::SpectateSlot::Stream,
            }]
        );
    }

    #[test]
    fn a_room_missing_from_the_lobby_is_not_streamed() {
        let mut app = testing::peek_app("me", &[("alpha", 10)]);
        testing::stream_room(&mut app, Some("missing"));
        app.update();
        assert_eq!(
            testing::drained_messages::<spectator::SpectateCommand>(&mut app),
            Vec::new()
        );
    }

    #[test]
    fn the_preview_prefers_a_member_who_is_not_a_player() {
        let dialed = testing::dialed_member_ids("spectator");
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
            testing::dialed_member_ids("spectator-a"),
            testing::dialed_member_ids("spectator-b")
        );
    }

    #[test]
    fn the_same_spectator_always_dials_the_same_member() {
        assert_eq!(
            testing::dialed_member_ids("spectator-a"),
            testing::dialed_member_ids("spectator-a")
        );
    }

    #[test]
    fn the_preview_never_dials_itself() {
        let dialed = testing::dialed_member_ids("member-c");
        assert_ne!(dialed, vec![String::from("member-c")]);
        let mut solo = testing::peek_app("solo", &[("alpha", 10)]);
        testing::add_member(
            &mut solo,
            "alpha",
            &testing::room_member(
                "solo",
                None,
                discovery::RoomMembership::HistoryHolder,
                &["/ip4/127.0.0.1/tcp/4001"],
            ),
        );
        testing::select_room(&mut solo, Some("alpha"));
        solo.update();
        assert_eq!(
            testing::drained_messages::<p2p::Libp2pCommand>(&mut solo),
            Vec::new(),
            "there is nobody else to dial"
        );
    }

    #[test]
    fn a_live_spectated_zone_creates_its_target() {
        let mut app = testing::peek_app("me", &[]);
        app.world_mut().spawn(testing::spectated_zone(
            spectator::SpectateSlot::Preview,
            "alpha",
            spectator::SpectateFreshness::Live,
        ));
        app.world_mut().spawn(testing::spectated_zone(
            spectator::SpectateSlot::Stream,
            "alpha",
            spectator::SpectateFreshness::Live,
        ));
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
        let mut app = testing::peek_app("me", &[]);
        app.world_mut().spawn(testing::spectated_zone(
            spectator::SpectateSlot::Preview,
            "alpha",
            spectator::SpectateFreshness::Live,
        ));
        app.world_mut().spawn(testing::spectated_zone(
            spectator::SpectateSlot::Stream,
            "alpha",
            spectator::SpectateFreshness::Live,
        ));
        app.update();
        testing::despawn_spectated(&mut app);
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
        let mut app = testing::peek_app("me", &[]);
        app.world_mut().spawn(testing::spectated_zone(
            spectator::SpectateSlot::Preview,
            "alpha",
            spectator::SpectateFreshness::Live,
        ));
        app.world_mut().spawn(testing::spectated_zone(
            spectator::SpectateSlot::Stream,
            "alpha",
            spectator::SpectateFreshness::Live,
        ));
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
        testing::despawn_spectated(&mut app);
        app.world_mut().spawn(testing::spectated_zone(
            spectator::SpectateSlot::Preview,
            "beta",
            spectator::SpectateFreshness::Live,
        ));
        app.world_mut().spawn(testing::spectated_zone(
            spectator::SpectateSlot::Stream,
            "beta",
            spectator::SpectateFreshness::Live,
        ));
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
