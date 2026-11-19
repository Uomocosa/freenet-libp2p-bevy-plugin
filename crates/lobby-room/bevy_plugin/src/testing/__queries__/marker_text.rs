use bevy::ecs::component::Component;
use bevy::ecs::world::World;
use bevy::prelude::Text;

#[must_use]
pub fn marker_text<C: Component>(world: &mut World) -> Option<String> {
    let mut query = world.query_filtered::<&Text, bevy::ecs::query::With<C>>();
    query
        .iter(world)
        .next()
        .map(|text| text.as_str().to_string())
}

#[cfg(test)]
mod tests {
    use bevy::prelude::{Name, Text, Transform, World};

    use super::marker_text;

    #[test]
    fn test_usage() {
        let mut world = World::new();
        world.spawn((Name::new("label"), Text::new("hello")));
        assert_eq!(marker_text::<Name>(&mut world), Some(String::from("hello")));
        assert_eq!(marker_text::<Transform>(&mut world), None);
    }
}
