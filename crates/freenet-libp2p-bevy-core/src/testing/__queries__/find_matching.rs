use bevy::ecs::entity::Entity;
use bevy::ecs::query::{QueryData, QueryFilter};
use bevy::ecs::world::World;

#[must_use]
pub fn find_matching<D: QueryData, F: QueryFilter>(world: &mut World) -> Option<Entity> {
    let mut query = world.query_filtered::<(Entity, D), F>();
    query.iter(world).next().map(|(entity, _)| entity)
}

#[cfg(test)]
mod tests {
    use bevy::prelude::{Transform, With, World};

    use super::find_matching;

    #[test]
    fn test_usage() {
        let mut world = World::new();
        let entity = world.spawn(Transform::default()).id();
        assert_eq!(
            find_matching::<&Transform, With<Transform>>(&mut world),
            Some(entity)
        );
    }
}
