use bevy::prelude::*;

use crate::peek;
use crate::replica_sync;
use crate::spectator;
use crate::ui;

use ui::discovery::{RoomStreamImage, RoomStreamPlaceholder};

type StreamNodes<'w, 's> = Query<
    'w,
    's,
    (&'static mut Visibility, Option<&'static mut ImageNode>),
    Or<(With<RoomStreamImage>, With<RoomStreamPlaceholder>)>,
>;

pub fn show_stream_image<S: replica_sync::traits::ReplicatedState>(
    spectated: Query<&spectator::Spectated<S>>,
    target: Option<Res<peek::RoomStreamTarget>>,
    mut nodes: StreamNodes,
    mut shown_image: Local<Option<Handle<Image>>>,
) {
    let is_showing_image = spectated
        .iter()
        .find(|spectated| spectated.slot == spectator::SpectateSlot::Stream)
        .is_some_and(|spectated| {
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
    if *shown_image == wanted_image {
        return;
    }
    (*shown_image).clone_from(&wanted_image);
    for (mut visibility, image_node) in &mut nodes {
        match (image_node, wanted_image.as_ref()) {
            (Some(mut image_node), Some(image)) => {
                if image_node.image != *image {
                    image_node.image = image.clone();
                }
                *visibility = Visibility::Visible;
            }
            (Some(_), None) | (None, Some(_)) => *visibility = Visibility::Hidden,
            (None, None) => *visibility = Visibility::Inherited,
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::show_stream_image;
    use crate::identity;
    use crate::peek;
    use crate::replica_sync;
    use crate::spectator;
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

    fn stream_image() -> Image {
        Image::new_fill(
            bevy::render::render_resource::Extent3d {
                width: 160,
                height: 100,
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
        let handle = app
            .world_mut()
            .resource_mut::<Assets<Image>>()
            .add(stream_image());
        app.insert_resource(peek::RoomStreamTarget {
            zone: identity::ZoneName::from("alpha"),
            image: handle,
        });
        app.add_systems(Update, show_stream_image::<State>);
        (app, image, placeholder)
    }

    fn spectate(app: &mut App, freshness: spectator::SpectateFreshness) {
        app.world_mut().spawn(spectator::Spectated::<State> {
            slot: spectator::SpectateSlot::Stream,
            zone: identity::ZoneName::from("alpha"),
            state: State,
            freshness,
        });
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

    fn visibility(app: &App, entity: Entity) -> Visibility {
        *app.world()
            .get::<Visibility>(entity)
            .expect("node visibility")
    }

    #[test]
    fn test_usage() {
        let (mut app, image, placeholder) = app();
        spectate(&mut app, spectator::SpectateFreshness::Live);
        app.update();
        assert!(app.world().get::<ImageNode>(image).is_some());
        assert_eq!(visibility(&app, image), Visibility::Visible);
        assert_eq!(visibility(&app, placeholder), Visibility::Hidden);
    }

    #[test]
    fn a_bootstrapping_stream_keeps_the_placeholder() {
        let (mut app, image, placeholder) = app();
        spectate(&mut app, spectator::SpectateFreshness::Bootstrapping);
        app.update();
        assert_eq!(visibility(&app, image), Visibility::Hidden);
        assert_eq!(visibility(&app, placeholder), Visibility::Inherited);
    }

    #[test]
    fn leaving_the_streamed_zone_restores_the_placeholder() {
        let (mut app, image, placeholder) = app();
        spectate(&mut app, spectator::SpectateFreshness::Live);
        app.update();
        despawn_spectated(&mut app);
        app.update();
        assert_eq!(visibility(&app, image), Visibility::Hidden);
        assert_eq!(visibility(&app, placeholder), Visibility::Inherited);
    }
}

#[cfg(test)]
mod png_tests {
    use super::show_stream_image;
    use crate::discovery as core_discovery;
    use crate::identity;
    use crate::peek;
    use crate::replica_sync;
    use crate::spectator;
    use crate::ui;
    use bevy::prelude::*;

    #[derive(Default, Resource)]
    struct PreviewState;

    impl replica_sync::traits::ReplicatedState for PreviewState {
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

    fn preview_row(name: &str) -> core_discovery::LobbyRow {
        core_discovery::LobbyRow {
            name: core_discovery::RoomName::from(name),
            occupants: 2,
            room_size: core_discovery::RoomSize::try_from(4).expect("4"),
            occupancy: core_discovery::RoomOccupancy::HasSpace,
            friends: 0,
        }
    }

    fn preview_build(app: &mut App) {
        app.init_resource::<Assets<Image>>();
        app.insert_resource(ui::discovery::StreamDisplay::default());
        app.insert_resource(ui::discovery::StreamedRoom(Some(
            core_discovery::RoomName::from("alpha"),
        )));
        app.insert_resource(core_discovery::LobbyRows(vec![preview_row("alpha")]));
        app.add_systems(Startup, ui::discovery::bevy_systems::spawn_room_stream);
        app.add_systems(
            Update,
            (
                ui::discovery::bevy_systems::refresh_room_stream,
                ui::discovery::bevy_systems::layout_stream_panel,
                ui::discovery::bevy_systems::fit_stream_image,
                show_stream_image::<PreviewState>,
            )
                .chain(),
        );
    }

    const fn preview_noop(_world: &mut World) {}

    fn preview_live(world: &mut World) {
        let image = world.resource_mut::<Assets<Image>>().add(Image::new_fill(
            bevy::render::render_resource::Extent3d {
                width: 160,
                height: 100,
                depth_or_array_layers: 1,
            },
            bevy::render::render_resource::TextureDimension::D2,
            &[48, 140, 168, 255],
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            bevy::asset::RenderAssetUsages::default(),
        ));
        let zone = identity::ZoneName::from("alpha");
        world.insert_resource(peek::RoomStreamTarget {
            zone: zone.clone(),
            image,
        });
        world.spawn(spectator::Spectated::<PreviewState> {
            slot: spectator::SpectateSlot::Stream,
            zone,
            state: PreviewState,
            freshness: spectator::SpectateFreshness::Live,
        });
    }

    #[test]
    fn show_stream_image_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("show_stream_image"),
            kind: lele_bevy_preview::scene::Kind::Plugin,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("placeholder"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("live image"),
                    apply: preview_live,
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
