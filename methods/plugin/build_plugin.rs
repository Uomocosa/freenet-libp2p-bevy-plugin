use std::sync::Mutex;

use bevy::prelude::*;
use tracing::error;

use crate::discovery;
use crate::room_sync;
#[cfg(feature = "testing")]
use crate::testing;

pub fn build_plugin(plugin: &discovery::Plugin, app: &mut App) {
    app.init_resource::<discovery::Status>();
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
    app.add_systems(
        Update,
        discovery::bevy_systems::project_status_to_entities
            .after(discovery::bevy_systems::drain_status)
            .in_set(room_sync::RoomSyncSet::Input),
    );
    app.add_systems(
        Update,
        discovery::bevy_systems::leave_failed_room.after(room_sync::RoomSyncSet::Drive),
    );
    app.add_systems(Last, discovery::bevy_systems::send_quit_on_exit);

    let (status_tx, status_rx) = tokio::sync::mpsc::unbounded_channel();
    let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
    let (command_tx, command_rx) = tokio::sync::mpsc::unbounded_channel();
    app.insert_resource(discovery::StatusReceiver(Mutex::new(Some(status_rx))));
    app.insert_resource(discovery::EventReceiver(Mutex::new(Some(event_rx))));
    app.insert_resource(discovery::CommandSender(command_tx));

    let Some(swarm) = discovery::Plugin::swarm_channels(app) else {
        error!(target: "discovery", "discovery: p2p resources missing, discovery disabled");
        return;
    };
    #[cfg(feature = "testing")]
    if let Some(task_override) = app
        .world_mut()
        .remove_resource::<discovery::LobbyTaskOverride>()
    {
        let endpoint = app
            .world()
            .get_resource::<discovery::FreenetPort>()
            .copied()
            .unwrap_or(discovery::FreenetPort(0));
        let _ = endpoint;
        app.add_systems(
            Update,
            discovery::bevy_systems::follow_room.after(discovery::bevy_systems::drain_events),
        );
        let channels = discovery::RunChannels {
            swarm,
            commands: command_rx,
            statuses: status_tx,
            events: event_tx,
        };
        let config = (**plugin).clone();
        let game = config.game.clone();
        let (target_tx, target_rx) = tokio::sync::watch::channel(Vec::new());
        let (lobby_tx, lobbies) = tokio::sync::mpsc::unbounded_channel();
        let (peers_tx, peers) = tokio::sync::mpsc::unbounded_channel();
        let handle = task_override.spawn_fake_lobby_task(
            discovery::LobbyTaskChannels {
                heartbeat_targets: target_rx,
                lobbies: lobby_tx,
                freenet_peers: peers_tx,
            },
            game,
        );
        if let Some(mut handles) = app
            .world_mut()
            .get_resource_mut::<testing::FakeTaskHandles>()
        {
            handles.lobby = Some(handle);
        }
        tokio::spawn(async move {
            config
                .run_event_loop(channels, target_tx, lobbies, peers)
                .await;
        });
        return;
    }
    let Some(endpoint) = app
        .world()
        .get_resource::<discovery::FreenetPort>()
        .copied()
    else {
        error!(target: "discovery", "discovery: FreenetPort missing, discovery disabled");
        return;
    };
    app.add_systems(
        Update,
        discovery::bevy_systems::follow_room.after(discovery::bevy_systems::drain_events),
    );
    let channels = discovery::RunChannels {
        swarm,
        commands: command_rx,
        statuses: status_tx,
        events: event_tx,
    };
    let config = (**plugin).clone();
    tokio::spawn(async move { config.run(endpoint, channels).await });
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
