use bevy::prelude::*;

use crate::identity;
use crate::peers;

pub fn get_or_spawn(
    index: &mut peers::PeerIndex,
    commands: &mut Commands,
    id: &identity::PeerId,
) -> Entity {
    if let Some(entity) = index.get(id).copied() {
        return entity;
    }
    let entity = commands.spawn(peers::Peer::from(id.clone())).id();
    index.insert(id.clone(), entity);
    entity
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::identity;
    use crate::peers;

    fn spawned_twice(mut commands: Commands, mut index: ResMut<peers::PeerIndex>) {
        let id = identity::PeerId::from("alice");
        let first = index.get_or_spawn(&mut commands, &id);
        let second = index.get_or_spawn(&mut commands, &id);
        assert_eq!(first, second);
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<peers::PeerIndex>();
        app.add_systems(Update, spawned_twice);
        app.update();
        let id = identity::PeerId::from("alice");
        let entity = app
            .world()
            .resource::<peers::PeerIndex>()
            .get(&id)
            .copied()
            .expect("indexed");
        assert_eq!(
            app.world()
                .get::<peers::Peer>(entity)
                .expect("peer")
                .as_str(),
            "alice"
        );
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<peers::Peer>>()
                .iter(app.world())
                .count(),
            1
        );
    }
}
