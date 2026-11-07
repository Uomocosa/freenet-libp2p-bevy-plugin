use std::sync::Mutex;

use bevy::prelude::*;

use crate::p2p;
use crate::plugin;
#[cfg(feature = "testing")]
use crate::testing;

pub fn build_plugin<Payload: p2p::Message>(p2p_plugin: &plugin::P2PPlugin<Payload>, app: &mut App) {
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    let (libp2p_tx, libp2p_rx) = tokio::sync::mpsc::unbounded_channel();
    let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
    #[cfg(feature = "testing")]
    if let Some(task_override) = app
        .world_mut()
        .remove_resource::<p2p::SwarmTaskOverride<Payload>>()
    {
        let game = p2p_plugin.game.clone();
        let handle = task_override.spawn_fake_swarm_task(cmd_rx, libp2p_rx, event_tx, game);
        if let Some(mut handles) = app
            .world_mut()
            .get_resource_mut::<testing::FakeTaskHandles>()
        {
            handles.swarm = Some(handle);
        }
        app.insert_resource(p2p::Events::<Payload>::default());
        app.insert_resource(p2p::Commands::<Payload>::default());
        app.insert_resource(p2p::PendingLibp2pCommands::default());
        app.insert_resource(p2p::ReadyChannel::default());
        app.insert_resource(p2p::Libp2pEventChannel::default());
        app.insert_resource(p2p::CommandSender(cmd_tx));
        app.insert_resource(p2p::EventReceiver(Mutex::new(Some(event_rx))));
        app.insert_resource(p2p::Libp2pCommandSender(libp2p_tx));
        app.add_systems(Update, p2p::bevy_systems::poll_p2p_events::<Payload>);
        return;
    }
    let keypair = p2p_plugin.runner_keypair();
    let config = p2p::RunnerConfig {
        mode: p2p_plugin.mode,
        mdns: p2p_plugin.mdns,
        game: p2p_plugin.game.clone(),
        gossipsub: p2p_plugin.gossipsub,
    };
    let _runner = config.spawn_runner::<Payload>(cmd_rx, libp2p_rx, event_tx, keypair);
    app.insert_resource(p2p::Events::<Payload>::default());
    app.insert_resource(p2p::Commands::<Payload>::default());
    app.insert_resource(p2p::PendingLibp2pCommands::default());
    app.insert_resource(p2p::ReadyChannel::default());
    app.insert_resource(p2p::Libp2pEventChannel::default());
    app.insert_resource(p2p::CommandSender(cmd_tx));
    app.insert_resource(p2p::EventReceiver(Mutex::new(Some(event_rx))));
    app.insert_resource(p2p::Libp2pCommandSender(libp2p_tx));
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
                .get_resource::<p2p::Libp2pCommandSender>()
                .is_some()
        );
    }
}
