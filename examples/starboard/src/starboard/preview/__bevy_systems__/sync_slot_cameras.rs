use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::lobby_room::peek;

use crate::starboard;

pub fn sync_slot_cameras(
    mut commands: Commands,
    preview_target: Option<Res<peek::RoomPreviewTarget>>,
    stream_target: Option<Res<peek::RoomStreamTarget>>,
    cameras: Query<(Entity, &starboard::preview::SlotCamera)>,
) {
    let [preview, stream] = starboard::preview::SLOT_RENDERS;
    let targets = [
        (preview, preview_target.map(|target| target.image.clone())),
        (stream, stream_target.map(|target| target.image.clone())),
    ];
    for (config, image) in targets {
        let existing = cameras
            .iter()
            .find(|(_, camera)| ***camera == config.slot)
            .map(|(entity, _)| entity);
        match (image, existing) {
            (None, Some(entity)) => {
                commands.entity(entity).despawn();
            }
            (Some(image), None) => {
                commands.spawn((
                    starboard::preview::SlotCamera(config.slot),
                    Camera2d,
                    Camera {
                        order: config.camera_order,
                        clear_color: ClearColorConfig::Custom(config.backdrop),
                        ..default()
                    },
                    RenderTarget::Image(image.into()),
                    RenderLayers::layer(config.layer),
                ));
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::asset::RenderAssetUsages;
    use bevy::camera::RenderTarget;
    use bevy::camera::visibility::RenderLayers;
    use bevy::prelude::*;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    use freenet_libp2p_bevy_plugin::identity;
    use freenet_libp2p_bevy_plugin::lobby_room::{peek, spectator};

    use super::sync_slot_cameras;
    use crate::starboard;

    fn target_image(app: &mut App, width: u32, height: u32) -> Handle<Image> {
        app.world_mut()
            .resource_mut::<Assets<Image>>()
            .add(Image::new_fill(
                Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                &[0, 0, 0, 255],
                TextureFormat::Bgra8UnormSrgb,
                RenderAssetUsages::default(),
            ))
    }

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default_nearest(),
        ));
        app.add_systems(Update, sync_slot_cameras);
        app
    }

    fn camera(
        app: &mut App,
        slot: spectator::SpectateSlot,
    ) -> Option<(RenderTarget, RenderLayers, isize)> {
        let world = app.world_mut();
        let mut cameras = world.query::<(
            &starboard::preview::SlotCamera,
            &RenderTarget,
            &RenderLayers,
            &Camera,
        )>();
        cameras
            .iter(world)
            .find(|(camera, ..)| ***camera == slot)
            .map(|(_, target, layers, camera)| (target.clone(), layers.clone(), camera.order))
    }

    fn start_preview(app: &mut App, image: Handle<Image>) {
        app.insert_resource(peek::RoomPreviewTarget {
            zone: identity::ZoneName::from("alpha"),
            image,
        });
    }

    fn start_stream(app: &mut App, image: Handle<Image>) {
        app.insert_resource(peek::RoomStreamTarget {
            zone: identity::ZoneName::from("beta"),
            image,
        });
    }

    #[test]
    fn test_usage() {
        let mut app = app();
        let image = target_image(&mut app, 320, 200);
        start_preview(&mut app, image.clone());
        app.update();

        let (target, layers, order) =
            camera(&mut app, spectator::SpectateSlot::Preview).expect("a preview camera");
        assert_eq!(target.as_image(), Some(&image));
        assert_eq!(
            layers,
            RenderLayers::layer(starboard::preview::PREVIEW_LAYER)
        );
        assert_eq!(order, starboard::preview::PREVIEW_CAMERA_ORDER);
    }

    #[test]
    fn each_slot_gets_its_own_camera_and_layer() {
        let mut app = app();
        let preview = target_image(&mut app, 320, 200);
        let stream = target_image(&mut app, 1280, 800);
        start_preview(&mut app, preview.clone());
        start_stream(&mut app, stream.clone());
        app.update();

        let (preview_target, preview_layers, preview_order) =
            camera(&mut app, spectator::SpectateSlot::Preview).expect("a preview camera");
        assert_eq!(preview_target.as_image(), Some(&preview));
        assert_eq!(
            preview_layers,
            RenderLayers::layer(starboard::preview::PREVIEW_LAYER)
        );
        assert_eq!(preview_order, starboard::preview::PREVIEW_CAMERA_ORDER);

        let (stream_target, stream_layers, stream_order) =
            camera(&mut app, spectator::SpectateSlot::Stream).expect("a stream camera");
        assert_eq!(stream_target.as_image(), Some(&stream));
        assert_eq!(
            stream_layers,
            RenderLayers::layer(starboard::preview::STREAM_LAYER)
        );
        assert_eq!(stream_order, starboard::preview::STREAM_CAMERA_ORDER);
    }

    #[test]
    fn dropping_one_target_leaves_the_other_camera() {
        let mut app = app();
        let preview = target_image(&mut app, 320, 200);
        let stream = target_image(&mut app, 1280, 800);
        start_preview(&mut app, preview);
        start_stream(&mut app, stream);
        app.update();

        app.world_mut().remove_resource::<peek::RoomPreviewTarget>();
        app.update();

        assert!(
            camera(&mut app, spectator::SpectateSlot::Preview).is_none(),
            "the preview camera goes with its target"
        );
        assert!(
            camera(&mut app, spectator::SpectateSlot::Stream).is_some(),
            "the stream camera stays"
        );
    }

    #[test]
    fn dropping_the_stream_target_despawns_its_camera() {
        let mut app = app();
        let stream = target_image(&mut app, 1280, 800);
        start_stream(&mut app, stream);
        app.update();
        assert!(camera(&mut app, spectator::SpectateSlot::Stream).is_some());

        app.world_mut().remove_resource::<peek::RoomStreamTarget>();
        app.update();
        assert!(camera(&mut app, spectator::SpectateSlot::Stream).is_none());
    }
}
