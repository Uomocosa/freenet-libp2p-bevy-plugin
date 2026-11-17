use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::lobby_room::peek;

use crate::starboard;

pub fn sync_preview_camera(
    mut commands: Commands,
    target: Option<Res<peek::RoomPreviewTarget>>,
    cameras: Query<Entity, With<starboard::preview::PreviewCamera>>,
) {
    let Some(target) = target else {
        for camera in &cameras {
            commands.entity(camera).despawn();
        }
        return;
    };
    if !cameras.is_empty() {
        return;
    }
    commands.spawn((
        starboard::preview::PreviewCamera,
        Camera2d,
        Camera {
            order: starboard::preview::PREVIEW_CAMERA_ORDER,
            clear_color: ClearColorConfig::Custom(starboard::preview::PREVIEW_BACKDROP_COLOR),
            ..default()
        },
        RenderTarget::Image(target.image.clone().into()),
        RenderLayers::layer(starboard::preview::PREVIEW_LAYER),
    ));
}

#[cfg(test)]
mod tests {
    use bevy::asset::RenderAssetUsages;
    use bevy::camera::RenderTarget;
    use bevy::camera::visibility::RenderLayers;
    use bevy::prelude::*;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    use freenet_libp2p_bevy_plugin::identity;
    use freenet_libp2p_bevy_plugin::lobby_room::peek;

    use super::sync_preview_camera;
    use crate::starboard;

    fn target_image(app: &mut App) -> Handle<Image> {
        app.world_mut()
            .resource_mut::<Assets<Image>>()
            .add(Image::new_fill(
                Extent3d {
                    width: 320,
                    height: 200,
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
        app.add_systems(Update, sync_preview_camera);
        app
    }

    fn camera(app: &mut App) -> Option<(Entity, RenderTarget, RenderLayers, isize)> {
        let world = app.world_mut();
        let mut cameras = world.query_filtered::<
            (Entity, &RenderTarget, &RenderLayers, &Camera),
            With<starboard::preview::PreviewCamera>,
        >();
        cameras
            .iter(world)
            .next()
            .map(|(entity, target, layers, camera)| {
                (entity, target.clone(), layers.clone(), camera.order)
            })
    }

    fn camera_entity(app: &mut App) -> Option<Entity> {
        let world = app.world_mut();
        let mut cameras = world.query_filtered::<Entity, With<starboard::preview::PreviewCamera>>();
        cameras.iter(world).next()
    }

    #[test]
    fn test_usage() {
        let mut app = app();
        let image = target_image(&mut app);
        app.insert_resource(peek::RoomPreviewTarget {
            zone: identity::ZoneName::from("alpha"),
            image: image.clone(),
        });
        app.update();

        let (_, target, layers, order) = camera(&mut app).expect("one preview camera");
        assert_eq!(
            target.as_image(),
            Some(&image),
            "the camera renders into the preview target"
        );
        assert_eq!(
            layers,
            RenderLayers::layer(starboard::preview::PREVIEW_LAYER)
        );
        assert_eq!(order, starboard::preview::PREVIEW_CAMERA_ORDER);
    }

    #[test]
    fn dropping_the_preview_target_despawns_the_camera() {
        let mut app = app();
        let image = target_image(&mut app);
        app.insert_resource(peek::RoomPreviewTarget {
            zone: identity::ZoneName::from("alpha"),
            image,
        });
        app.update();
        assert!(camera(&mut app).is_some());

        app.world_mut().remove_resource::<peek::RoomPreviewTarget>();
        app.update();
        assert!(camera(&mut app).is_none());
    }

    #[test]
    fn an_existing_camera_is_kept_across_frames() {
        let mut app = app();
        let image = target_image(&mut app);
        app.insert_resource(peek::RoomPreviewTarget {
            zone: identity::ZoneName::from("alpha"),
            image,
        });
        app.update();
        let first = camera_entity(&mut app).expect("one preview camera");

        app.update();
        assert_eq!(camera_entity(&mut app).expect("one preview camera"), first);
    }
}
