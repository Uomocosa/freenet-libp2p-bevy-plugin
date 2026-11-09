use std::sync::Mutex;

use bevy::prelude::*;
use tracing::error;

use crate::discovery;
#[cfg(feature = "testing")]
use crate::testing;

pub fn spawn_lobby_task(plugin: &discovery::Plugin, app: &mut App) {
    let Some(swarm) = discovery::Plugin::swarm_channels(app) else {
        error!(target: "discovery", "discovery: p2p resources missing, discovery disabled");
        return;
    };
    let (target_tx, target_rx) = tokio::sync::watch::channel(Vec::new());
    let (lobby_tx, lobby_rx) = tokio::sync::mpsc::unbounded_channel();
    let (peers_tx, peers_rx) = tokio::sync::mpsc::unbounded_channel();
    app.insert_resource(discovery::HeartbeatTargetSender(target_tx));
    app.insert_resource(discovery::LobbyUpdateReceiver(Mutex::new(Some(lobby_rx))));
    app.insert_resource(discovery::FreenetPeersReceiver(Mutex::new(Some(peers_rx))));
    #[cfg(feature = "testing")]
    if let Some(task_override) = app
        .world_mut()
        .remove_resource::<discovery::LobbyTaskOverride>()
    {
        let game = plugin.game.clone();
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
    let config = (**plugin).clone();
    tokio::spawn(async move {
        config
            .run(endpoint, swarm, target_rx, lobby_tx, peers_tx)
            .await;
    });
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::spawn_lobby_task;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let plugin = discovery::Plugin::new(discovery::Config::default());
        spawn_lobby_task(&plugin, &mut app);
        assert!(
            app.world()
                .get_resource::<discovery::LobbyUpdateReceiver>()
                .is_none()
        );
    }
}
