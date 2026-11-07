use crate::discovery;
use crate::identity;
use discovery::state_machine::{DiscoveryState, Input};

pub async fn run_event_loop(
    config: &discovery::Config,
    channels: discovery::RunChannels,
    heartbeat_targets: tokio::sync::watch::Sender<Vec<discovery::state_machine::HeartbeatTarget>>,
    mut lobbies: tokio::sync::mpsc::UnboundedReceiver<discovery::Lobby>,
    mut freenet_peers: tokio::sync::mpsc::UnboundedReceiver<Vec<identity::FreenetPeer>>,
) {
    let discovery::RunChannels {
        mut swarm,
        mut commands,
        statuses,
        events,
    } = channels;
    let Some(ready) = identity::Peer::wait_ready(&mut swarm.ready).await else {
        return;
    };
    let me = identity::Peer {
        addrs: identity::PeerAddr::dialable(ready.addrs),
        id: ready.id,
    };
    let mut state = DiscoveryState::new(me);
    state.display_name = config.display_name.clone();
    state.lease_limits = discovery::LobbyLeaseLimits {
        lease_ttl: config.lease.ttl,
        remote_clock_skew_margin: config.lease.remote_clock_skew_margin,
    };
    state.game = config.game.clone();
    state.history_size_limits = config.history_size_limits;
    let mut interval = tokio::time::interval(discovery::TICK);
    loop {
        let input = tokio::select! {
            Some(command) = commands.recv() => Input::Command(command),
            Ok(event) = swarm.events.recv() => Input::Libp2pEvent(event),
            Some(lobby) = lobbies.recv() => Input::LobbyUpdated(lobby),
            Some(connected) = freenet_peers.recv() => Input::FreenetPeers(connected),
            Ok(()) = swarm.observed.changed() => {
                let Some(addrs) = swarm.observed.borrow_and_update().clone() else {
                    continue;
                };
                Input::OwnAddrsChanged(identity::PeerAddr::dialable(addrs))
            }
            _ = interval.tick() => Input::Tick,
        };
        let outputs = state.update(input, discovery::UnixTime::now());
        discovery::Config::flush(outputs, &swarm.commands, &events);
        let target = state.heartbeat_targets();
        heartbeat_targets.send_if_modified(|current| {
            let changed = *current != target;
            if changed {
                current.clone_from(&target);
            }
            changed
        });
        let _ = statuses.send(state.status());
    }
}

#[cfg(test)]
mod tests {
    use super::run_event_loop;
    use crate::discovery;

    #[tokio::test]
    async fn test_usage() {
        let config = discovery::Config::default();
        let (libp2p_commands, _libp2p_rx) = tokio::sync::mpsc::unbounded_channel();
        let (libp2p_events, _libp2p_event_rx) = tokio::sync::broadcast::channel(16);
        let (_ready_tx, ready_rx) = tokio::sync::watch::channel(None);
        let (_observed_tx, observed_rx) = tokio::sync::watch::channel(None);
        let (_commands_tx, commands_rx) = tokio::sync::mpsc::unbounded_channel();
        let (statuses_tx, _statuses_rx) = tokio::sync::mpsc::unbounded_channel();
        let (events_tx, _events_rx) = tokio::sync::mpsc::unbounded_channel();
        let channels = discovery::RunChannels {
            swarm: discovery::libp2p::SwarmChannels {
                commands: libp2p_commands,
                events: libp2p_events.subscribe(),
                ready: ready_rx,
                observed: observed_rx,
            },
            commands: commands_rx,
            statuses: statuses_tx,
            events: events_tx,
        };
        let (target_tx, _target_rx) = tokio::sync::watch::channel(Vec::new());
        let (_lobbies_tx, lobbies_rx) = tokio::sync::mpsc::unbounded_channel();
        let (_peers_tx, peers_rx) = tokio::sync::mpsc::unbounded_channel();
        let handle = tokio::spawn(async move {
            run_event_loop(&config, channels, target_tx, lobbies_rx, peers_rx).await;
        });
        handle.abort();
    }
}
