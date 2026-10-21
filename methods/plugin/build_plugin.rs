use std::sync::Mutex;

use bevy::prelude::*;
use tracing::error;

use crate::discovery;
use crate::p2p;

pub fn build_plugin(plugin: &discovery::Plugin, app: &mut App) {
    app.init_resource::<discovery::Status>();
    app.insert_resource(plugin.lease);
    app.add_message::<discovery::Command>();
    app.add_message::<discovery::Event>();
    app.add_systems(
        Update,
        (
            discovery::bevy_systems::drain_status,
            discovery::bevy_systems::drain_events,
            discovery::bevy_systems::forward_commands,
        ),
    );
    app.add_systems(Last, discovery::bevy_systems::send_quit_on_exit);

    let (status_tx, status_rx) = tokio::sync::mpsc::unbounded_channel();
    let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
    let (command_tx, command_rx) = tokio::sync::mpsc::unbounded_channel();
    app.insert_resource(discovery::StatusReceiver(Mutex::new(Some(status_rx))));
    app.insert_resource(discovery::EventReceiver(Mutex::new(Some(event_rx))));
    app.insert_resource(discovery::CommandSender(command_tx));

    let Some(net) = net_link(app) else {
        error!(target: "discovery", "discovery: p2p resources missing, discovery disabled");
        return;
    };
    let Some(endpoint) = app
        .world()
        .get_resource::<discovery::FreenetPort>()
        .copied()
    else {
        error!(target: "discovery", "discovery: FreenetPort missing, discovery disabled");
        return;
    };
    if plugin.auto_subscribe == discovery::AutoSubscribe::Enabled {
        app.add_systems(
            Update,
            discovery::bevy_systems::follow_room.after(discovery::bevy_systems::drain_events),
        );
    }
    let channels = discovery::RunChannels {
        net,
        commands: command_rx,
        statuses: status_tx,
        events: event_tx,
    };
    tokio::spawn(discovery::run((**plugin).clone(), endpoint, channels));
}

fn net_link(app: &App) -> Option<discovery::libp2p::NetLink> {
    let events = app
        .world()
        .get_resource::<p2p::NetEventChannel>()?
        .subscribe();
    let commands = (**app.world().get_resource::<p2p::NetCommandSender>()?).clone();
    let (ready, observed) = app.world().get_resource::<p2p::ReadyChannel>()?.subscribe();
    Some(discovery::libp2p::NetLink {
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
        assert!(app.world().get_resource::<discovery::Status>().is_some());
    }
}
