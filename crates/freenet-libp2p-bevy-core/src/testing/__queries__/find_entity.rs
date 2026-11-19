use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::With;
use bevy::ecs::world::World;

#[must_use]
pub fn find_entity<C: Component>(world: &mut World) -> Option<Entity> {
    let mut query = world.query_filtered::<Entity, With<C>>();
    query.iter(world).next()
}

#[cfg(test)]
mod tests {
    use bevy::prelude::{Name, Transform, World};

    use super::find_entity;

    #[test]
    fn test_usage() {
        let mut world = World::new();
        let entity = world.spawn(Transform::default()).id();
        assert_eq!(find_entity::<Transform>(&mut world), Some(entity));
        assert_eq!(find_entity::<Name>(&mut world), None);
    }
}
