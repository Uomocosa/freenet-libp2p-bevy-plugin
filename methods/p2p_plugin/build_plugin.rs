use std::sync::Mutex;

use bevy::prelude::*;

use crate::p2p;
use crate::plugin;

pub fn build_plugin<Payload: p2p::Message>(p2p_plugin: &plugin::P2PPlugin<Payload>, app: &mut App) {
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    let (net_tx, net_rx) = tokio::sync::mpsc::unbounded_channel();
    let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
    let keypair = libp2p::identity::Keypair::generate_ed25519();
    let config = p2p::RunnerConfig {
        mode: p2p_plugin.mode,
        mdns: p2p_plugin.mdns,
        game: p2p_plugin.game.clone(),
        gossipsub: p2p_plugin.gossipsub,
    };
    let _runner = config.spawn_runner::<Payload>(cmd_rx, net_rx, event_tx, keypair);
    app.insert_resource(p2p::Events::<Payload>::default());
    app.insert_resource(p2p::Commands::<Payload>::default());
    app.insert_resource(p2p::PendingNetCommands::default());
    app.insert_resource(p2p::ReadyChannel::default());
    app.insert_resource(p2p::NetEventChannel::default());
    app.insert_resource(p2p::CommandSender(cmd_tx));
    app.insert_resource(p2p::EventReceiver(Mutex::new(Some(event_rx))));
    app.insert_resource(p2p::NetCommandSender(net_tx));
    app.add_systems(Update, p2p::bevy_systems::poll_p2p_events::<Payload>);
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use serde::{Deserialize, Serialize};

    use super::build_plugin;
    use crate::p2p;
    use crate::plugin;
    use derive_more::Deref;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Deref)]
    struct Dummy(u32);

    #[tokio::test]
    async fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        build_plugin(
            &plugin::P2PPlugin(plugin::Config::<Dummy>::new(
                p2p::TransportMode::Both,
                p2p::MdnsMode::Disabled,
            )),
            &mut app,
        );
        app.update();
        assert!(app.world().get_resource::<p2p::Events<Dummy>>().is_some());
        assert!(
            app.world()
                .get_resource::<p2p::CommandSender<Dummy>>()
                .is_some()
        );
        assert!(
            app.world()
                .get_resource::<p2p::NetCommandSender>()
                .is_some()
        );
    }
}
