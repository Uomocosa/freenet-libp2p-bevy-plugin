use tokio::sync::mpsc::UnboundedSender;

use crate::discovery;
use crate::identity;
use crate::p2p;
use discovery::state_machine::{DiscoveryState, Input, Output};

pub async fn run(
    config: &discovery::Config,
    endpoint: discovery::FreenetPort,
    channels: discovery::RunChannels,
) {
    let discovery::RunChannels {
        mut net,
        mut commands,
        statuses,
        events,
    } = channels;
    let Some(ready) = identity::Peer::wait_ready(&mut net.ready).await else {
        return;
    };
    let me = identity::Peer {
        addrs: identity::PeerAddr::dialable(ready.addrs),
        id: ready.id,
    };
    let mut state = DiscoveryState::new(me.clone());
    state.display_name = config.display_name.clone();
    state.lease_ttl = config.lease.ttl;
    state.game = config.game.clone();
    state.history_size_limits = config.history_size_limits;
    let (target_tx, target_rx) = tokio::sync::watch::channel(Vec::new());
    let (lobby_tx, mut lobbies) = tokio::sync::mpsc::unbounded_channel();
    let (peers_tx, mut peers) = tokio::sync::mpsc::unbounded_channel();
    let freenet_config = discovery::freenet::Config {
        host: "127.0.0.1".to_string(),
        port: *endpoint,
        me: me.id.clone(),
        params: config.game.contract_params(),
        discovery: config.clone(),
    };
    let peers_config = freenet_config.clone();
    tokio::spawn(async move { peers_config.poll_peers(peers_tx).await });
    tokio::spawn(async move {
        let client = freenet_config.connect_retry(&target_rx).await;
        freenet_config.run_lobby(client, target_rx, lobby_tx).await;
    });
    let mut interval = tokio::time::interval(discovery::TICK);
    loop {
        let input = tokio::select! {
            Some(command) = commands.recv() => Input::Command(command),
            Ok(event) = net.events.recv() => Input::NetEvent(event),
            Some(lobby) = lobbies.recv() => Input::LobbyUpdated(lobby),
            Some(connected) = peers.recv() => Input::FreenetPeers(connected),
            Ok(()) = net.observed.changed() => {
                let Some(addrs) = net.observed.borrow_and_update().clone() else {
                    continue;
                };
                Input::OwnAddrsChanged(identity::PeerAddr::dialable(addrs))
            }
            _ = interval.tick() => Input::Tick,
        };
        let outputs = state.update(input, discovery::UnixTime::now());
        flush(outputs, &net.commands, &events);
        let target = state.heartbeat_targets();
        target_tx.send_if_modified(|current| {
            let changed = *current != target;
            if changed {
                current.clone_from(&target);
            }
            changed
        });
        let _ = statuses.send(state.status());
    }
}

// needed helper: performs the outputs of one update
fn flush(
    outputs: Vec<Output>,
    net: &UnboundedSender<p2p::NetCommand>,
    events: &UnboundedSender<discovery::Event>,
) {
    for output in outputs {
        match output {
            Output::Notify(event) => {
                let _ = events.send(event);
            }
            Output::NetCommand(command) => {
                let _ = net.send(command);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::run;

    use crate::discovery;

    #[tokio::test]
    async fn test_usage() {
        let config = discovery::Config::default();
        let (net_commands, _net_rx) = tokio::sync::mpsc::unbounded_channel();
        let (net_events, _net_event_rx) = tokio::sync::broadcast::channel(16);
        let (_ready_tx, ready_rx) = tokio::sync::watch::channel(None);
        let (_observed_tx, observed_rx) = tokio::sync::watch::channel(None);
        let (_commands_tx, commands_rx) = tokio::sync::mpsc::unbounded_channel();
        let (statuses_tx, _statuses_rx) = tokio::sync::mpsc::unbounded_channel();
        let (events_tx, _events_rx) = tokio::sync::mpsc::unbounded_channel();
        let channels = discovery::RunChannels {
            net: discovery::libp2p::NetLink {
                commands: net_commands,
                events: net_events.subscribe(),
                ready: ready_rx,
                observed: observed_rx,
            },
            commands: commands_rx,
            statuses: statuses_tx,
            events: events_tx,
        };
        let handle = tokio::spawn(async move {
            run(&config, discovery::FreenetPort(0), channels).await;
        });
        handle.abort();
    }
}
