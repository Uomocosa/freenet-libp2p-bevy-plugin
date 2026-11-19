use bevy::ecs::entity::Entity;
use bevy::ecs::world::World;

use crate::ui;

#[must_use]
pub fn room_row(world: &mut World, room: &str) -> Option<Entity> {
    let mut rows = world.query::<(Entity, &ui::discovery::RoomButton)>();
    rows.iter(world)
        .find(|(_entity, button)| button.as_str() == room)
        .map(|(entity, _button)| entity)
}

#[cfg(test)]
mod tests {
    use bevy::prelude::World;

    use crate::discovery;
    use crate::ui;

    use super::room_row;

    #[test]
    fn test_usage() {
        let mut world = World::new();
        let alpha = world
            .spawn(ui::discovery::RoomButton(discovery::RoomName::from(
                "alpha",
            )))
            .id();
        world.spawn(ui::discovery::RoomButton(discovery::RoomName::from("beta")));
        assert_eq!(room_row(&mut world, "alpha"), Some(alpha));
        assert_eq!(room_row(&mut world, "gamma"), None);
    }
}
