use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;

use crate::peek;
use crate::replica_sync;
use crate::spectator;

pub fn sync_preview_target<S: replica_sync::traits::ReplicatedState>(
    mut commands: Commands,
    images: Option<ResMut<Assets<Image>>>,
    spectated: Query<&spectator::Spectated<S>>,
    target: Option<ResMut<peek::RoomPreviewTarget>>,
) {
    let spectated = spectated
        .iter()
        .find(|spectated| spectated.slot == spectator::SpectateSlot::Preview);
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
