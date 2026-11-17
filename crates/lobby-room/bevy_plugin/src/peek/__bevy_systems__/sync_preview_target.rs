use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;

use crate::peek;
use crate::replica_sync;
use crate::spectator;

pub fn sync_preview_target<S: replica_sync::traits::ReplicatedState>(
    mut commands: Commands,
    images: Option<ResMut<Assets<Image>>>,
    spectated: Option<Res<spectator::Spectated<S>>>,
    target: Option<ResMut<peek::RoomPreviewTarget>>,
) {
    let spectated = spectated.map(Res::into_inner);
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
                peek::PREVIEW_TARGET_WIDTH,
                peek::PREVIEW_TARGET_HEIGHT,
                TextureFormat::Bgra8UnormSrgb,
                None,
            ));
            commands.insert_resource(peek::RoomPreviewTarget {
                zone: spectated.zone.clone(),
                image,
            });
        }
        (None, Some(target)) => {
            if let Some(mut images) = images {
                let _ = images.remove(target.image.id());
            }
            commands.remove_resource::<peek::RoomPreviewTarget>();
        }
        (None, None) => {}
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::sync_preview_target;
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
            zone: identity::ZoneName::from(zone),
            state: State,
            freshness,
        }
    }

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Image>>();
        app.add_systems(Update, sync_preview_target::<State>);
        app
    }

    fn target(app: &App) -> Option<&peek::RoomPreviewTarget> {
        app.world().get_resource::<peek::RoomPreviewTarget>()
    }

    #[test]
    fn test_usage() {
        let mut app = app();
        app.insert_resource(spectated("alpha", spectator::SpectateFreshness::Live));
        app.update();
        let target = target(&app).expect("preview target");
        assert_eq!(target.zone, identity::ZoneName::from("alpha"));
        assert_eq!(app.world().resource::<Assets<Image>>().len(), 1);
    }

    #[test]
    fn leaving_the_spectated_zone_removes_the_target_and_its_image() {
        let mut app = app();
        app.insert_resource(spectated("alpha", spectator::SpectateFreshness::Live));
        app.update();
        app.world_mut()
            .remove_resource::<spectator::Spectated<State>>();
        app.update();
        assert!(target(&app).is_none());
        assert_eq!(app.world().resource::<Assets<Image>>().len(), 0);
    }

    #[test]
    fn switching_rooms_keeps_the_image_and_updates_the_zone() {
        let mut app = app();
        app.insert_resource(spectated("alpha", spectator::SpectateFreshness::Live));
        app.update();
        let image = target(&app).expect("preview target").image.clone();
        app.insert_resource(spectated(
            "beta",
            spectator::SpectateFreshness::Bootstrapping,
        ));
        app.update();
        let target = target(&app).expect("preview target");
        assert_eq!(target.zone, identity::ZoneName::from("beta"));
        assert_eq!(target.image, image);
        assert_eq!(app.world().resource::<Assets<Image>>().len(), 1);
    }
}
