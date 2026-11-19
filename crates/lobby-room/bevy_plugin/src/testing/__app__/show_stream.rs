use bevy::prelude::World;

use crate::discovery;
use crate::spectator;
use crate::testing;
use crate::ui;

pub fn show_stream(world: &mut World, room: &str) {
    **world.resource_mut::<ui::discovery::StreamedRoom>() = Some(discovery::RoomName::from(room));
    testing::spectate_room(
        world,
        spectator::SpectateSlot::Stream,
        room,
        spectator::SpectateFreshness::Live,
    );
}

#[cfg(test)]
mod tests {
    use bevy::prelude::{App, Assets, Image};

    use crate::discovery;
    use crate::ui;

    use super::show_stream;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.init_resource::<Assets<Image>>();
        app.init_resource::<ui::discovery::StreamedRoom>();
        show_stream(app.world_mut(), "alpha");
        assert_eq!(
            **app.world().resource::<ui::discovery::StreamedRoom>(),
            Some(discovery::RoomName::from("alpha"))
        );
    }
}
