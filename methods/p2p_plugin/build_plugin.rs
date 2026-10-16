use std::sync::Mutex;

use bevy::prelude::*;

use crate::p2p;
use crate::plugin;
use crate::roster;

pub fn build_plugin<T: p2p::Message>(p2p_plugin: &plugin::P2PPlugin<T>, app: &mut App) {
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    let (net_tx, net_rx) = tokio::sync::mpsc::unbounded_channel();
    let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
    let keypair = libp2p::identity::Keypair::generate_ed25519();
    let _runner = p2p::spawn_runner::<T>(
        cmd_rx,
        net_rx,
        event_tx,
        keypair,
        p2p_plugin.mode,
        p2p_plugin.mdns,
    );
    app.insert_resource(p2p::Events::<T>::default());
    app.insert_resource(p2p::Commands::<T>::default());
    app.insert_resource(p2p::PendingNetCommands::default());
    app.insert_resource(p2p::ReadyChannel::default());
    app.insert_resource(p2p::NetEventChannel::default());
    app.insert_resource(roster::RoomRoster::default());
    app.insert_resource(roster::ActiveRoom::default());
    app.insert_resource(p2p::CommandChannel(cmd_tx));
    app.insert_resource(p2p::EventChannel(Mutex::new(Some(event_rx))));
    app.insert_resource(p2p::NetCommandChannel(net_tx));
    app.add_systems(
        Update,
        (
            p2p::bevy_systems::poll_p2p_events::<T>,
            roster::bevy_systems::poll_roster::<T>,
        )
            .chain(),
    );
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use serde::{Deserialize, Serialize};

    use super::build_plugin;
    use crate::identity;
    use crate::p2p;
    use crate::plugin;
    use crate::roster;
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
                .get_resource::<p2p::CommandChannel<Dummy>>()
                .is_some()
        );
        assert!(
            app.world()
                .get_resource::<p2p::NetCommandChannel>()
                .is_some()
        );
        assert!(app.world().get_resource::<roster::RoomRoster>().is_some());
        assert_eq!(
            app.world().get_resource::<roster::ActiveRoom>(),
            Some(&roster::ActiveRoom(identity::RoomName::from("default")))
        );
    }
}
