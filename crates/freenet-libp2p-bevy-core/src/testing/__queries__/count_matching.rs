use bevy::ecs::query::{QueryData, QueryFilter};
use bevy::ecs::world::World;

#[must_use]
pub fn count_matching<D: QueryData, F: QueryFilter>(world: &mut World) -> usize {
    let mut query = world.query_filtered::<D, F>();
    query.iter(world).count()
}

#[cfg(test)]
mod tests {
    use bevy::prelude::{Transform, With, World};

    use super::count_matching;

    #[test]
    fn test_usage() {
        let mut world = World::new();
        world.spawn(Transform::default());
        world.spawn(Transform::default());
        assert_eq!(count_matching::<&Transform, With<Transform>>(&mut world), 2);
    }
}
