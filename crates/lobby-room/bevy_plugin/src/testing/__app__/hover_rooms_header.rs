use bevy::picking::hover::Hovered;
use bevy::prelude::{Entity, With, World};

use crate::testing;
use crate::ui;

pub fn hover_rooms_header(world: &mut World) {
    let mut indicators = world.query_filtered::<Entity, With<ui::discovery::RoomSearchIndicator>>();
    let indicators: Vec<Entity> = indicators.iter(world).collect();
    for entity in indicators {
        world
            .entity_mut(entity)
            .insert((Hovered(true), testing::PinHovered));
    }
}

#[cfg(test)]
mod tests {
    use bevy::picking::hover::Hovered;
    use bevy::prelude::World;

    use crate::ui;

    use super::hover_rooms_header;

    #[test]
    fn test_usage() {
        let mut world = World::new();
        let indicator = world.spawn(ui::discovery::RoomSearchIndicator).id();
        hover_rooms_header(&mut world);
        assert!(world.get::<Hovered>(indicator).is_some_and(Hovered::get));
    }
}
