use bevy::picking::hover::Hovered;
use bevy::prelude::World;

use crate::discovery;
use crate::spectator;
use crate::testing;
use crate::ui;

pub fn show_room_preview(world: &mut World, room: &str) {
    **world.resource_mut::<ui::discovery::SelectedRoom>() = Some(discovery::RoomName::from(room));
    if let Some(row) = testing::room_row(world, room) {
        world
            .entity_mut(row)
            .insert((Hovered(true), testing::PinHovered));
    }
    testing::spectate_room(
        world,
        spectator::SpectateSlot::Preview,
        room,
        spectator::SpectateFreshness::Live,
    );
}

#[cfg(test)]
mod tests {
    use bevy::prelude::{App, Assets, Image};

    use crate::discovery;
    use crate::ui;

    use super::show_room_preview;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.init_resource::<Assets<Image>>();
        app.init_resource::<ui::discovery::SelectedRoom>();
        show_room_preview(app.world_mut(), "alpha");
        assert_eq!(
            **app.world().resource::<ui::discovery::SelectedRoom>(),
            Some(discovery::RoomName::from("alpha"))
        );
    }
}
