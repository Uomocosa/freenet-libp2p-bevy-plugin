use bevy::ecs::component::Component;
use bevy::ecs::world::World;

#[must_use]
pub fn component<C: Component + Clone>(world: &mut World) -> Option<C> {
    let mut query = world.query::<&C>();
    query.iter(world).next().cloned()
}

#[cfg(test)]
mod tests {
    use bevy::prelude::{Transform, World};

    use super::component;

    #[test]
    fn test_usage() {
        let mut world = World::new();
        world.spawn(Transform::default());
        assert_eq!(
            component::<Transform>(&mut world),
            Some(Transform::default())
        );
    }
}
