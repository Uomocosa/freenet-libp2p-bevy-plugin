use bevy::prelude::*;

use crate::discovery;
use crate::p2p;

pub fn start_discovery_when_swarm_ready(
    mut commands: Commands,
    ready: Option<Res<p2p::ReadyChannel>>,
    config: Option<Res<discovery::Config>>,
) {
    let Some(ready) = ready.map(Res::into_inner) else {
        return;
    };
    let Some(peer) = ready.ready_rx.borrow().clone() else {
        return;
    };
    let Some(config) = config.map(Res::into_inner) else {
        return;
    };
    let state = discovery::state_machine::DiscoveryState::from_ready_peer(peer, config);
    commands.insert_resource(state);
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::start_discovery_when_swarm_ready;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;

    fn ready_with(id: &str) -> p2p::ReadyChannel {
        let channel = p2p::ReadyChannel::default();
        channel.ready_tx.send_replace(Some(identity::Peer {
            id: identity::PeerId::from(id),
            addrs: vec![identity::PeerAddr::from("/ip4/1.2.3.4/tcp/4001")],
        }));
        channel
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ready_with("me"));
        app.insert_resource(discovery::Config::default());
        app.add_systems(Update, start_discovery_when_swarm_ready);
        app.update();
        let state = app
            .world()
            .get_resource::<discovery::state_machine::DiscoveryState>()
            .expect("state inserted");
        assert_eq!(state.me.id, identity::PeerId::from("me"));
        assert_eq!(
            state.me.addrs,
            vec![identity::PeerAddr::from("/ip4/1.2.3.4/tcp/4001")]
        );
    }

    #[test]
    fn no_spawn_before_ready() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::ReadyChannel::default());
        app.insert_resource(discovery::Config::default());
        app.add_systems(Update, start_discovery_when_swarm_ready);
        app.update();
        assert!(
            app.world()
                .get_resource::<discovery::state_machine::DiscoveryState>()
                .is_none()
        );
    }

    #[test]
    fn state_is_spawned_once() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ready_with("me"));
        app.insert_resource(discovery::Config::default());
        app.add_systems(
            Update,
            start_discovery_when_swarm_ready.run_if(not(resource_exists::<
                discovery::state_machine::DiscoveryState,
            >)),
        );
        app.update();
        app.world_mut()
            .resource_mut::<discovery::state_machine::DiscoveryState>()
            .is_quitting = true;
        app.update();
        assert!(
            app.world()
                .resource::<discovery::state_machine::DiscoveryState>()
                .is_quitting
        );
    }
}
