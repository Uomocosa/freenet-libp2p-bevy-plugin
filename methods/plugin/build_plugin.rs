use std::sync::Mutex;

use bevy::prelude::*;
use tracing::error;

use crate::discovery;
use crate::p2p;

pub fn build_plugin(plugin: &discovery::Plugin, app: &mut App) {
    app.init_resource::<discovery::Snapshot>();
    app.add_message::<discovery::Command>();
    app.add_message::<discovery::Event>();
    app.add_systems(
        Update,
        (
            discovery::bevy_systems::drain_snapshots,
            discovery::bevy_systems::drain_events,
            discovery::bevy_systems::forward_commands,
        ),
    );

    let (snapshot_tx, snapshot_rx) = tokio::sync::mpsc::unbounded_channel();
    let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
    let (command_tx, command_rx) = tokio::sync::mpsc::unbounded_channel();
    app.insert_resource(discovery::SnapshotFeed(Mutex::new(Some(snapshot_rx))));
    app.insert_resource(discovery::EventFeed(Mutex::new(Some(event_rx))));
    app.insert_resource(discovery::CommandSender(command_tx));

    let Some(net) = net_link(app) else {
        error!(target: "room_lobby", "discovery: p2p resources missing, discovery disabled");
        return;
    };
    let Some(endpoint) = app
        .world()
        .get_resource::<discovery::FreenetPort>()
        .copied()
    else {
        error!(target: "room_lobby", "discovery: FreenetPort missing, discovery disabled");
        return;
    };
    let channels = discovery::Channels {
        net,
        commands: command_rx,
        snapshots: snapshot_tx,
        events: event_tx,
    };
    tokio::spawn(discovery::run((**plugin).clone(), endpoint, channels));
}

fn net_link(app: &mut App) -> Option<discovery::libp2p::Link> {
    let events = app
        .world_mut()
        .get_resource_mut::<p2p::EventTap>()
        .and_then(|tap| tap.take_rx())?;
    let commands = (**app.world().get_resource::<p2p::NetBridge>()?).clone();
    let (ready, observed) = app.world().get_resource::<p2p::Signals>()?.subscribe();
    Some(discovery::libp2p::Link {
        commands,
        events,
        ready,
        observed,
    })
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::build_plugin;
    use crate::discovery;

    #[tokio::test]
    async fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let plugin = discovery::Plugin::new(discovery::Config::default());
        build_plugin(&plugin, &mut app);
        assert!(app.world().get_resource::<discovery::Snapshot>().is_some());
    }
}
