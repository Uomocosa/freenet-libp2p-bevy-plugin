use crate::ui;
use bevy::prelude::*;

use ui::discovery::{RoomStreamImage, RoomStreamViewport, STREAM_IMAGE_ASPECT};

pub fn fit_stream_image(
    viewport: Option<Single<&ComputedNode, With<RoomStreamViewport>>>,
    image: Option<Single<&mut Node, With<RoomStreamImage>>>,
) {
    let (Some(viewport), Some(image)) = (viewport, image) else {
        return;
    };
    let viewport = viewport.into_inner();
    let padding = viewport.padding();
    let scale = viewport.inverse_scale_factor();
    let panel = Vec2::new(
        (viewport.size().x - padding.min_inset.x - padding.max_inset.x) * scale,
        (viewport.size().y - padding.min_inset.y - padding.max_inset.y) * scale,
    );
    if panel.x <= 0.0 || panel.y <= 0.0 {
        return;
    }
    let height = panel.y.min(panel.x / STREAM_IMAGE_ASPECT);
    let wanted_width = Val::Px(height * STREAM_IMAGE_ASPECT);
    let wanted_height = Val::Px(height);
    let mut node = image.into_inner();
    if node.width != wanted_width {
        node.width = wanted_width;
    }
    if node.height != wanted_height {
        node.height = wanted_height;
    }
}

#[cfg(test)]
mod png_tests {
    use crate::discovery as core_discovery;
    use crate::identity;
    use crate::peek;
    use crate::replica_sync;
    use crate::spectator;
    use crate::ui;
    use bevy::prelude::*;

    #[test]
    fn fit_stream_image_ui_png_preview() {
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

        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("fit_stream_image"),
            kind: lele_bevy_preview::scene::Kind::Plugin,
            build: |app: &mut App| {
                app.init_resource::<Assets<Image>>();
                app.insert_resource(ui::discovery::StreamDisplay::default());
                app.insert_resource(ui::discovery::StreamedRoom(Some(
                    core_discovery::RoomName::from("alpha"),
                )));
                app.insert_resource(core_discovery::LobbyRows(vec![core_discovery::LobbyRow {
                    name: core_discovery::RoomName::from("alpha"),
                    occupants: 2,
                    room_size: core_discovery::RoomSize::try_from(4).expect("4"),
                    occupancy: core_discovery::RoomOccupancy::HasSpace,
                    friends: 0,
                }]));
                app.add_systems(Startup, ui::discovery::bevy_systems::spawn_room_stream);
                app.add_systems(
                    Update,
                    (
                        ui::discovery::bevy_systems::refresh_room_stream,
                        ui::discovery::bevy_systems::layout_stream_panel,
                        ui::discovery::bevy_systems::fit_stream_image,
                        ui::discovery::bevy_systems::show_stream_image::<PreviewState>,
                    )
                        .chain(),
                );
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("placeholder"),
                    apply: |_world: &mut World| {},
                },
                lele_bevy_preview::scene::State {
                    label: String::from("fitted stream image"),
                    apply: |world: &mut World| {
                        let image = world.resource_mut::<Assets<Image>>().add(Image::new_fill(
                            bevy::render::render_resource::Extent3d {
                                width: 160,
                                height: 100,
                                depth_or_array_layers: 1,
                            },
                            bevy::render::render_resource::TextureDimension::D2,
                            &[82, 138, 176, 255],
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
                    },
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
