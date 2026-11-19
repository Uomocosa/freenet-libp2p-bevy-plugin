use bevy::ecs::component::Component;
use bevy::ecs::world::World;
use bevy::prelude::Visibility;

#[must_use]
pub fn marker_visibility<C: Component>(world: &mut World) -> Option<Visibility> {
    let mut query = world.query_filtered::<&Visibility, bevy::ecs::query::With<C>>();
    query.iter(world).next().copied()
}

#[cfg(test)]
mod tests {
    use bevy::prelude::{Name, Transform, Visibility, World};

    use super::marker_visibility;

    #[test]
    fn test_usage() {
        let mut world = World::new();
        world.spawn((Name::new("panel"), Visibility::Hidden));
        assert_eq!(
            marker_visibility::<Name>(&mut world),
            Some(Visibility::Hidden)
        );
        assert_eq!(marker_visibility::<Transform>(&mut world), None);
    }
}
