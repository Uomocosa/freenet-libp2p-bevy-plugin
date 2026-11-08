use bevy::prelude::*;

use crate::identity;
use crate::peers;

pub fn get_or_spawn(
    index: &mut peers::RoomIndex,
    commands: &mut Commands,
    name: &identity::RoomName,
) -> Entity {
    if let Some(entity) = index.get(name).copied() {
        return entity;
    }
    let entity = commands.spawn(peers::Room::from(name.clone())).id();
    index.insert(name.clone(), entity);
    entity
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::identity;
    use crate::peers;

    fn spawned_twice(mut commands: Commands, mut index: ResMut<peers::RoomIndex>) {
        let name = identity::RoomName::from("lobby");
        let first = index.get_or_spawn(&mut commands, &name);
        let second = index.get_or_spawn(&mut commands, &name);
        assert_eq!(first, second);
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<peers::RoomIndex>();
        app.add_systems(Update, spawned_twice);
        app.update();
        let name = identity::RoomName::from("lobby");
        let entity = app
            .world()
            .resource::<peers::RoomIndex>()
            .get(&name)
            .copied()
            .expect("indexed");
        assert_eq!(
            app.world()
                .get::<peers::Room>(entity)
                .expect("room")
                .as_str(),
            "lobby"
        );
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<peers::Room>>()
                .iter(app.world())
                .count(),
            1
        );
    }
}
