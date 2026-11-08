use bevy::prelude::*;

use crate::p2p;
use crate::peers;

pub fn spawn_me_on_ready(
    mut commands: Commands,
    ready: Option<Res<p2p::ReadyChannel>>,
    me_entities: Query<Entity, With<peers::Me>>,
    mut peer_index: Option<ResMut<peers::PeerIndex>>,
) {
    let Some(peer_index) = peer_index.as_deref_mut() else {
        return;
    };
    let snapshot = ready.and_then(|ready| ready.ready_rx.borrow().clone());
    let Some(me) = snapshot else {
        return;
    };
    if !me_entities.is_empty() {
        return;
    }
    let entity = peer_index.get_or_spawn(&mut commands, &me.id);
    commands.entity(entity).insert(peers::Me);
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::spawn_me_on_ready;
    use crate::identity;
    use crate::p2p;
    use crate::peers;

    fn ready_with(id: &str) -> p2p::ReadyChannel {
        let channel = p2p::ReadyChannel::default();
        channel.ready_tx.send_replace(Some(identity::Peer {
            id: identity::PeerId::from(id),
            addrs: Vec::new(),
        }));
        channel
    }

    fn me_entities(app: &mut App) -> Vec<(Entity, identity::PeerId)> {
        let mut found = app
            .world_mut()
            .query_filtered::<(Entity, &peers::Peer), With<peers::Me>>();
        found
            .iter(app.world())
            .map(|(entity, peer)| (entity, (**peer).clone()))
            .collect()
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ready_with("me"));
        app.init_resource::<peers::PeerIndex>();
        app.add_systems(Update, spawn_me_on_ready);
        app.update();
        let found = me_entities(&mut app);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].1, identity::PeerId::from("me"));
        assert_eq!(
            app.world()
                .resource::<peers::PeerIndex>()
                .get(&identity::PeerId::from("me")),
            Some(&found[0].0)
        );
    }

    #[test]
    fn no_spawn_before_ready() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::ReadyChannel::default());
        app.init_resource::<peers::PeerIndex>();
        app.add_systems(Update, spawn_me_on_ready);
        app.update();
        assert_eq!(me_entities(&mut app).len(), 0);
    }

    #[test]
    fn me_is_spawned_once() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ready_with("me"));
        app.init_resource::<peers::PeerIndex>();
        app.add_systems(Update, spawn_me_on_ready);
        app.update();
        app.update();
        assert_eq!(me_entities(&mut app).len(), 1);
    }
}
