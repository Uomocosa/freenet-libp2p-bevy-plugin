use bevy::prelude::*;

use crate::peek;
use crate::replica_sync;
use crate::spectator;
use crate::ui;

use ui::discovery::{RoomPreviewImage, RoomPreviewPlaceholder};

type PreviewImageNode<'w, 's> = Single<
    'w,
    's,
    (&'static mut ImageNode, &'static mut Visibility),
    (With<RoomPreviewImage>, Without<RoomPreviewPlaceholder>),
>;
type PreviewPlaceholderVisibility<'w, 's> = Single<
    'w,
    's,
    &'static mut Visibility,
    (With<RoomPreviewPlaceholder>, Without<RoomPreviewImage>),
>;

pub fn show_preview_image<S: replica_sync::traits::ReplicatedState>(
    spectated: Query<&spectator::Spectated<S>>,
    target: Option<Res<peek::RoomPreviewTarget>>,
    image: Option<PreviewImageNode>,
    placeholder: Option<PreviewPlaceholderVisibility>,
    mut shown_image: Local<Option<Handle<Image>>>,
) {
    let Some(image) = image else {
        return;
    };
    let spectated = spectated
        .iter()
        .find(|spectated| spectated.slot == spectator::SpectateSlot::Preview);
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
    if let Some(placeholder) = placeholder {
        let mut visibility = placeholder.into_inner();
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
    (*shown_image).clone_from(&wanted_image);
    let (mut image_node, mut visibility) = image.into_inner();
    if let Some(handle) = wanted_image.as_ref() {
        if image_node.image != *handle {
            image_node.image = handle.clone();
        }
        *visibility = Visibility::Visible;
    } else {
        *visibility = Visibility::Hidden;
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::show_preview_image;
    use crate::discovery as core_discovery;
    use crate::identity;
    use crate::peek;
    use crate::replica_sync;
    use crate::spectator;
    use crate::ui;

    #[test]
    fn show_preview_image_ui_png_preview() {
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

        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("show_preview_image"),
            kind: lele_bevy_preview::scene::Kind::Plugin,
            build: |app: &mut App| {
                app.insert_resource(core_discovery::LobbyRows(vec![core_discovery::LobbyRow {
                    name: core_discovery::RoomName::from("alpha"),
                    occupants: 2,
                    room_size: core_discovery::RoomSize::DEFAULT,
                    occupancy: core_discovery::RoomOccupancy::HasSpace,
                    friends: 1,
                }]));
                app.init_resource::<Assets<Image>>();
                app.insert_resource(ui::discovery::StreamedRoom::default());
                app.insert_resource(ui::discovery::SelectedRoom(Some(
                    core_discovery::RoomName::from("alpha"),
                )));
                app.add_systems(Startup, ui::discovery::bevy_systems::spawn_room_preview);
                app.add_systems(
                    Update,
                    (
                        ui::discovery::bevy_systems::refresh_room_preview,
                        show_preview_image::<State>,
                    ),
                );
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("placeholder"),
                    apply: |_world: &mut World| {},
                },
                lele_bevy_preview::scene::State {
                    label: String::from("spectating image"),
                    apply: |world: &mut World| {
                        let image = world.resource_mut::<Assets<Image>>().add(Image::new_fill(
                            bevy::render::render_resource::Extent3d {
                                width: 64,
                                height: 40,
                                depth_or_array_layers: 1,
                            },
                            bevy::render::render_resource::TextureDimension::D2,
                            &[48, 140, 168, 255],
                            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                            bevy::asset::RenderAssetUsages::default(),
                        ));
                        let zone = identity::ZoneName::from("alpha");
                        world.insert_resource(peek::RoomPreviewTarget {
                            zone: zone.clone(),
                            image,
                        });
                        world.spawn(spectator::Spectated::<State> {
                            slot: spectator::SpectateSlot::Preview,
                            zone,
                            state: State,
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
