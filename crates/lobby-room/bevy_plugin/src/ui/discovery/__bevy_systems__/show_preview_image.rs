use bevy::prelude::*;

use crate::peek;
use crate::replica_sync;
use crate::spectator;
use crate::ui;

use ui::discovery::{RoomPreviewMemberList, RoomPreviewViewport};

pub fn show_preview_image<S: replica_sync::traits::ReplicatedState>(
    mut commands: Commands,
    spectated: Option<Res<spectator::Spectated<S>>>,
    target: Option<Res<peek::RoomPreviewTarget>>,
    viewport: Option<Single<Entity, With<RoomPreviewViewport>>>,
    member_list: Option<Single<&mut Visibility, With<RoomPreviewMemberList>>>,
    mut shown_image: Local<Option<Handle<Image>>>,
) {
    let Some(viewport) = viewport else {
        return;
    };
    let spectated = spectated.map(Res::into_inner);
    let target = target.map(Res::into_inner);
    let is_showing_image = spectated.is_some_and(|spectated| {
        matches!(
            spectated.freshness,
            spectator::SpectateFreshness::Live | spectator::SpectateFreshness::Repairing
        )
    });
    let wanted_image = if is_showing_image {
        target.map(|target| target.image.clone())
    } else {
        None
    };
    if let Some(member_list) = member_list {
        let mut visibility = member_list.into_inner();
        let wanted_visibility = if is_showing_image {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        if *visibility != wanted_visibility {
            *visibility = wanted_visibility;
        }
    }
    if *shown_image == wanted_image {
        return;
    }
    let entity = viewport.into_inner();
    if let Some(image) = wanted_image.as_ref() {
        commands
            .entity(entity)
            .insert(ImageNode::new(image.clone()));
    } else {
        commands.entity(entity).remove::<ImageNode>();
    }
    *shown_image = wanted_image;
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use bevy::prelude::*;

    use super::show_preview_image;
    use crate::discovery as core_discovery;
    use crate::identity;
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

    fn preview_image() -> Image {
        Image::new_fill(
            bevy::render::render_resource::Extent3d {
                width: 64,
                height: 40,
                depth_or_array_layers: 1,
            },
            bevy::render::render_resource::TextureDimension::D2,
            &[48, 140, 168, 255],
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            bevy::asset::RenderAssetUsages::default(),
        )
    }

    fn app() -> (App, Entity, Entity) {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Image>>();
        let viewport = app
            .world_mut()
            .spawn((ui::discovery::RoomPreviewViewport, Node::default()))
            .id();
        let member_list = app
            .world_mut()
            .spawn((
                ui::discovery::RoomPreviewMemberList,
                Node::default(),
                ChildOf(viewport),
            ))
            .id();
        let image = app
            .world_mut()
            .resource_mut::<Assets<Image>>()
            .add(preview_image());
        app.insert_resource(peek::RoomPreviewTarget {
            zone: identity::ZoneName::from("alpha"),
            image,
        });
        app.add_systems(Update, show_preview_image::<State>);
        (app, viewport, member_list)
    }

    fn spectate(app: &mut App, freshness: spectator::SpectateFreshness) {
        app.insert_resource(spectator::Spectated::<State> {
            zone: identity::ZoneName::from("alpha"),
            state: State,
            freshness,
        });
    }

    fn member_visibility(app: &App, member_list: Entity) -> Visibility {
        *app.world()
            .get::<Visibility>(member_list)
            .expect("member list visibility")
    }

    #[test]
    fn test_usage() {
        let (mut app, viewport, member_list) = app();
        spectate(&mut app, spectator::SpectateFreshness::Live);
        app.update();
        assert!(app.world().get::<ImageNode>(viewport).is_some());
        assert_eq!(member_visibility(&app, member_list), Visibility::Hidden);
    }

    #[test]
    fn a_bootstrapping_preview_keeps_the_member_list() {
        let (mut app, viewport, member_list) = app();
        spectate(&mut app, spectator::SpectateFreshness::Bootstrapping);
        app.update();
        assert!(app.world().get::<ImageNode>(viewport).is_none());
        assert_eq!(member_visibility(&app, member_list), Visibility::Inherited);
    }

    #[test]
    fn a_repairing_preview_keeps_showing_the_image() {
        let (mut app, viewport, _member_list) = app();
        spectate(&mut app, spectator::SpectateFreshness::Repairing);
        app.update();
        assert!(app.world().get::<ImageNode>(viewport).is_some());
    }

    #[test]
    fn leaving_the_spectated_state_restores_the_member_list() {
        let (mut app, viewport, member_list) = app();
        spectate(&mut app, spectator::SpectateFreshness::Live);
        app.update();
        app.world_mut()
            .remove_resource::<spectator::Spectated<State>>();
        app.update();
        assert!(app.world().get::<ImageNode>(viewport).is_none());
        assert_eq!(member_visibility(&app, member_list), Visibility::Inherited);
    }

    fn preview_discovery_state() -> core_discovery::state_machine::DiscoveryState {
        let mut state = core_discovery::state_machine::DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        let members = ["alice", "bob"]
            .iter()
            .map(|name| {
                (
                    identity::PeerId::from(*name),
                    core_discovery::RoomMember {
                        peer: identity::Peer {
                            id: identity::PeerId::from(*name),
                            addrs: Vec::new(),
                        },
                        display_name: Some(identity::Username::from(*name)),
                        updated_at: time::UnixTime::EPOCH,
                        membership: core_discovery::RoomMembership::Occupant,
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        state.lobby.insert(
            core_discovery::RoomName::from("alpha"),
            core_discovery::Room {
                room_size: core_discovery::RoomSize::DEFAULT,
                created_at: time::UnixTime::EPOCH,
                members,
            },
        );
        state
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(core_discovery::LobbyRows(vec![core_discovery::LobbyRow {
            name: core_discovery::RoomName::from("alpha"),
            occupants: 2,
            room_size: core_discovery::RoomSize::DEFAULT,
            occupancy: core_discovery::RoomOccupancy::HasSpace,
            friends: 1,
        }]));
        app.insert_resource(ui::discovery::SelectedRoom {
            room: Some(core_discovery::RoomName::from("alpha")),
            is_locked: true,
        });
        app.insert_resource(preview_discovery_state());
        app.add_systems(Startup, ui::discovery::bevy_systems::spawn_room_preview);
        app.add_systems(
            Update,
            (
                ui::discovery::bevy_systems::refresh_room_preview,
                ui::discovery::bevy_systems::rebuild_room_preview_members,
                show_preview_image::<State>,
            ),
        );
    }

    const fn preview_noop(_world: &mut World) {}

    fn preview_spectate(world: &mut World) {
        let image = world.resource_mut::<Assets<Image>>().add(preview_image());
        let zone = identity::ZoneName::from("alpha");
        world.insert_resource(peek::RoomPreviewTarget {
            zone: zone.clone(),
            image,
        });
        world.insert_resource(spectator::Spectated::<State> {
            zone,
            state: State,
            freshness: spectator::SpectateFreshness::Live,
        });
    }

    #[test]
    fn show_preview_image_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("show_preview_image"),
            kind: lele_bevy_preview::scene::Kind::Plugin,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("member list"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("spectating image"),
                    apply: preview_spectate,
                },
            ],
            timeline: None,
        }
        .run(&lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }
}
