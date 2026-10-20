use tokio::sync::mpsc::UnboundedSender;

use crate::discovery;
use crate::identity;
use crate::p2p;
use discovery::state_machine::{Input, Output, State};

pub async fn run(
    config: discovery::Config,
    endpoint: discovery::FreenetPort,
    channels: discovery::RunChannels,
) {
    let discovery::RunChannels {
        mut net,
        mut commands,
        statuses,
        events,
    } = channels;
    let Some(ready) = discovery::libp2p::wait_ready(&mut net.ready).await else {
        return;
    };
    let me = identity::Peer {
        addrs: discovery::libp2p::dialable(ready.addrs),
        id: ready.id,
    };
    let mut state = State::new(me.clone());
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
        params: discovery::freenet::contract_params(&config.game),
        discovery: config,
    };
    tokio::spawn(discovery::freenet::poll_peers(
        freenet_config.clone(),
        peers_tx,
    ));
    tokio::spawn(async move {
        let client = discovery::freenet::connect_retry(&freenet_config, &target_rx).await;
        discovery::freenet::run_lobby(&freenet_config, client, target_rx, lobby_tx).await;
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
                Input::OwnAddrsChanged(discovery::libp2p::dialable(addrs))
            }
            _ = interval.tick() => Input::Tick,
        };
        let outputs =
            discovery::state_machine::update(&mut state, input, discovery::UnixTime::now());
        flush(outputs, &net.commands, &events);
        let target = discovery::state_machine::heartbeat_targets(&state);
        target_tx.send_if_modified(|current| {
            let changed = *current != target;
            if changed {
                current.clone_from(&target);
            }
            changed
        });
        let _ = statuses.send(discovery::state_machine::status(&state));
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

// no test_usage necessary
