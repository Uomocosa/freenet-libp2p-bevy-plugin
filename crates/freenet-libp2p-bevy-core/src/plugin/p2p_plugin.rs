use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::App;
use derive_more::Deref;

use crate::p2p;
use crate::peers;
use crate::plugin;
#[cfg(feature = "testing")]
use crate::testing;
use bevy::prelude::*;
use std::sync::Mutex;

#[derive(Deref)]
pub struct P2PPlugin<Payload: p2p::traits::Message>(pub plugin::Config<Payload>);

#[rustfmt::skip]
impl<Payload: p2p::traits::Message> bevy::prelude::Plugin for P2PPlugin<Payload> {
    fn build(&self, app: &mut App) {
        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
        let (libp2p_tx, libp2p_rx) = tokio::sync::mpsc::unbounded_channel();
        let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
        #[cfg(feature = "testing")]
        if let Some(task_override) = app
            .world_mut()
            .remove_resource::<p2p::SwarmTaskOverride<Payload>>()
        {
            let game = self.game.clone();
            let handle = task_override.spawn_fake_swarm_task(cmd_rx, libp2p_rx, event_tx, game);
            if let Some(mut handles) = app
                .world_mut()
                .get_resource_mut::<testing::FakeTaskHandles>()
            {
                handles.swarm = Some(handle);
            }
            app.insert_resource(p2p::PendingEvents::<Payload>::default());
            app.insert_resource(p2p::PendingCommands::<Payload>::default());
            app.add_message::<p2p::Libp2pCommand>();
            app.insert_resource(p2p::ReadyChannel::default());
            app.insert_resource(p2p::Libp2pEventChannel::default());
            app.insert_resource(p2p::CommandSender(cmd_tx));
            app.insert_resource(p2p::EventReceiver(Mutex::new(Some(event_rx))));
            app.insert_resource(p2p::Libp2pCommandSender(libp2p_tx));
            app.add_systems(Update, p2p::bevy_systems::poll_p2p_events::<Payload>);
            app.add_systems(Last, p2p::bevy_systems::send_libp2p_commands);
            app.add_plugins(peers::TrackingPlugin);
            return;
        }
        let keypair = self.runner_keypair();
        let config = p2p::RunnerConfig {
            mode: self.mode,
            mdns: self.mdns,
            game: self.game.clone(),
            gossipsub: self.gossipsub,
        };
        let _runner = config.spawn_runner::<Payload>(cmd_rx, libp2p_rx, event_tx, keypair);
        app.insert_resource(p2p::PendingEvents::<Payload>::default());
        app.insert_resource(p2p::PendingCommands::<Payload>::default());
        app.add_message::<p2p::Libp2pCommand>();
        app.insert_resource(p2p::ReadyChannel::default());
        app.insert_resource(p2p::Libp2pEventChannel::default());
        app.insert_resource(p2p::CommandSender(cmd_tx));
        app.insert_resource(p2p::EventReceiver(Mutex::new(Some(event_rx))));
        app.insert_resource(p2p::Libp2pCommandSender(libp2p_tx));
        app.add_systems(Update, p2p::bevy_systems::poll_p2p_events::<Payload>);
        app.add_systems(Last, p2p::bevy_systems::send_libp2p_commands);
        app.add_plugins(peers::TrackingPlugin);
    }
}

#[atomic_delegates]
impl<Payload: p2p::traits::Message> P2PPlugin<Payload> {}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use serde::{Deserialize, Serialize};

    use super::P2PPlugin;
    use crate::p2p;
    use crate::plugin;
    use derive_more::Deref;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Deref)]
    struct Dummy(u32);

    #[tokio::test]
    async fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(P2PPlugin(plugin::Config::<Dummy>::new(
            p2p::TransportMode::Both,
            p2p::MdnsMode::Disabled,
        )));
        app.update();
        assert!(
            app.world()
                .get_resource::<p2p::PendingEvents<Dummy>>()
                .is_some()
        );
        assert!(
            app.world()
                .get_resource::<Messages<p2p::Libp2pCommand>>()
                .is_some()
        );
        assert!(
            app.world()
                .get_resource::<p2p::Libp2pEventChannel>()
                .is_some()
        );
        assert!(app.world().get_resource::<p2p::ReadyChannel>().is_some());
    }
}

#[cfg(test)]
mod build_plugin_tests {
    use bevy::prelude::Plugin;
    use bevy::prelude::*;
    use serde::{Deserialize, Serialize};

    use crate::p2p;
    use crate::plugin;
    use derive_more::Deref;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Deref)]
    struct Dummy(u32);

    #[tokio::test]
    async fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        Plugin::build(
            &plugin::P2PPlugin(plugin::Config::<Dummy>::new(
                p2p::TransportMode::Both,
                p2p::MdnsMode::Disabled,
            )),
            &mut app,
        );
        app.update();
        assert!(
            app.world()
                .get_resource::<p2p::PendingEvents<Dummy>>()
                .is_some()
        );
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
