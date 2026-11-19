use bevy::prelude::{Assets, Image, World};

use crate::identity;
use crate::peek;
use crate::spectator;
use crate::testing;

pub fn spectate_room(
    world: &mut World,
    slot: spectator::SpectateSlot,
    room: &str,
    freshness: spectator::SpectateFreshness,
) {
    let image = world
        .resource_mut::<Assets<Image>>()
        .add(testing::test_pattern_image());
    let zone = identity::ZoneName::from(room);
    match slot {
        spectator::SpectateSlot::Preview => {
            world.insert_resource(peek::RoomPreviewTarget { zone, image });
        }
        spectator::SpectateSlot::Stream => {
            world.insert_resource(peek::RoomStreamTarget { zone, image });
        }
    }
    world.spawn(testing::spectated_zone(slot, room, freshness));
}

#[cfg(test)]
mod tests {
    use bevy::prelude::{App, Assets, Image};

    use crate::identity;
    use crate::peek;
    use crate::spectator;
    use crate::testing;

    use super::spectate_room;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.init_resource::<Assets<Image>>();
        spectate_room(
            app.world_mut(),
            spectator::SpectateSlot::Preview,
            "alpha",
            spectator::SpectateFreshness::Live,
        );
        assert_eq!(
            app.world().resource::<peek::RoomPreviewTarget>().zone,
            identity::ZoneName::from("alpha")
        );
        assert!(
            testing::find_entity::<spectator::Spectated<testing::FakeReplica<u8, u8>>>(
                app.world_mut()
            )
            .is_some()
        );
    }
}
