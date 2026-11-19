use bevy::prelude::{Entity, With, World};

use crate::peek;
use crate::spectator;
use crate::testing;
use crate::ui;

pub fn clear_room_targets(world: &mut World) {
    let mut query =
        world.query_filtered::<Entity, With<spectator::Spectated<testing::FakeReplica<u8, u8>>>>();
    let entities: Vec<Entity> = query.iter(world).collect();
    for entity in entities {
        world.despawn(entity);
    }
    world.remove_resource::<peek::RoomPreviewTarget>();
    world.remove_resource::<peek::RoomStreamTarget>();
    **world.resource_mut::<ui::discovery::SelectedRoom>() = None;
    **world.resource_mut::<ui::discovery::StreamedRoom>() = None;
    *world.resource_mut::<ui::discovery::StreamDisplay>() = ui::discovery::StreamDisplay::Panel;
}

#[cfg(test)]
mod tests {
    use bevy::prelude::{App, Assets, Image};

    use crate::spectator;
    use crate::testing;
    use crate::ui;

    use super::clear_room_targets;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.init_resource::<Assets<Image>>();
        app.init_resource::<ui::discovery::SelectedRoom>();
        app.init_resource::<ui::discovery::StreamedRoom>();
        app.init_resource::<ui::discovery::StreamDisplay>();
        testing::spectate_room(
            app.world_mut(),
            spectator::SpectateSlot::Preview,
            "alpha",
            spectator::SpectateFreshness::Live,
        );
        clear_room_targets(app.world_mut());
        assert!(
            testing::find_entity::<spectator::Spectated<testing::FakeReplica<u8, u8>>>(
                app.world_mut()
            )
            .is_none()
        );
        assert_eq!(
            **app.world().resource::<ui::discovery::SelectedRoom>(),
            None
        );
        assert_eq!(
            *app.world().resource::<ui::discovery::StreamDisplay>(),
            ui::discovery::StreamDisplay::Panel
        );
    }
}
