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
