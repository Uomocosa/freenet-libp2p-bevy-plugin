use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;

use crate::peek;
use crate::replica_sync;
use crate::spectator;

pub fn sync_stream_target<S: replica_sync::traits::ReplicatedState>(
    mut commands: Commands,
    images: Option<ResMut<Assets<Image>>>,
    spectated: Query<&spectator::Spectated<S>>,
    target: Option<ResMut<peek::RoomStreamTarget>>,
) {
    let spectated = spectated
        .iter()
        .find(|spectated| spectated.slot == spectator::SpectateSlot::Stream);
    match (spectated, target) {
        (Some(spectated), Some(mut target)) => {
            if target.zone != spectated.zone {
                target.zone = spectated.zone.clone();
            }
        }
        (Some(spectated), None) => {
            let Some(mut images) = images else {
                return;
            };
            let image = images.add(Image::new_target_texture(
                peek::STREAM_TARGET_WIDTH,
                peek::STREAM_TARGET_HEIGHT,
                TextureFormat::Bgra8UnormSrgb,
                None,
            ));
            commands.insert_resource(peek::RoomStreamTarget {
                zone: spectated.zone.clone(),
                image,
            });
        }
        (None, Some(target)) => {
            if let Some(mut images) = images {
                let _ = images.remove(target.image.id());
            }
            commands.remove_resource::<peek::RoomStreamTarget>();
        }
        (None, None) => {}
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::sync_stream_target;
    use crate::identity;
    use crate::peek;
    use crate::replica_sync;
    use crate::spectator;

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

    fn spectated(
        zone: &str,
        freshness: spectator::SpectateFreshness,
    ) -> spectator::Spectated<State> {
        spectator::Spectated {
            slot: spectator::SpectateSlot::Stream,
            zone: identity::ZoneName::from(zone),
            state: State,
            freshness,
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

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Image>>();
        app.add_systems(Update, sync_stream_target::<State>);
        app
    }

    fn target(app: &App) -> Option<&peek::RoomStreamTarget> {
        app.world().get_resource::<peek::RoomStreamTarget>()
    }

    #[test]
    fn test_usage() {
        let mut app = app();
        app.world_mut()
            .spawn(spectated("alpha", spectator::SpectateFreshness::Live));
        app.update();
        let target = target(&app).expect("stream target");
        assert_eq!(target.zone, identity::ZoneName::from("alpha"));
        assert_eq!(app.world().resource::<Assets<Image>>().len(), 1);
    }

    #[test]
    fn the_stream_target_matches_the_board_aspect() {
        let mut app = app();
        app.world_mut()
            .spawn(spectated("alpha", spectator::SpectateFreshness::Live));
        app.update();
        let target = target(&app).expect("stream target");
        let size = app
            .world()
            .resource::<Assets<Image>>()
            .get(&target.image)
            .expect("stream image")
            .size();
        assert_eq!(size.x, peek::STREAM_TARGET_WIDTH);
        assert_eq!(size.y, peek::STREAM_TARGET_HEIGHT);
        assert_eq!(size.x * 10, size.y * 16);
    }

    #[test]
    fn leaving_the_streamed_zone_removes_the_target_and_its_image() {
        let mut app = app();
        app.world_mut()
            .spawn(spectated("alpha", spectator::SpectateFreshness::Live));
        app.update();
        despawn_spectated(&mut app);
        app.update();
        assert!(target(&app).is_none());
        assert_eq!(app.world().resource::<Assets<Image>>().len(), 0);
    }

    #[test]
    fn switching_rooms_keeps_the_image_and_updates_the_zone() {
        let mut app = app();
        app.world_mut()
            .spawn(spectated("alpha", spectator::SpectateFreshness::Live));
        app.update();
        let image = target(&app).expect("stream target").image.clone();
        despawn_spectated(&mut app);
        app.world_mut().spawn(spectated(
            "beta",
            spectator::SpectateFreshness::Bootstrapping,
        ));
        app.update();
        let target = target(&app).expect("stream target");
        assert_eq!(target.zone, identity::ZoneName::from("beta"));
        assert_eq!(target.image, image);
        assert_eq!(app.world().resource::<Assets<Image>>().len(), 1);
    }
}
