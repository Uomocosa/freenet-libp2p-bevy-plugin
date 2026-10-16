use std::time::Duration;

use tokio::sync::mpsc::UnboundedSender;

use crate::discovery;
use crate::identity;
use crate::p2p;
use discovery::state_machine::{Input, Output, State};

pub async fn run(
    config: discovery::Config,
    endpoint: discovery::FreenetPort,
    channels: discovery::Channels,
) {
    let discovery::Channels {
        mut net,
        mut commands,
        snapshots,
        events,
    } = channels;
    let Some(ready) = discovery::libp2p::wait_ready(&mut net.ready).await else {
        return;
    };
    let me = identity::Peer {
        addrs: discovery::libp2p::dialable(ready.addrs),
        id: ready.id,
    };
    let timing = config.timing;
    let mut state = State::new(me.clone(), timing);
    state.display_name = config.display_name.clone();
    let (target_tx, target_rx) = tokio::sync::watch::channel(None);
    let (lobby_tx, mut lobbies) = tokio::sync::mpsc::unbounded_channel();
    let (peers_tx, mut peers) = tokio::sync::mpsc::unbounded_channel();
    let freenet_config = discovery::freenet::FreenetConfig {
        host: "127.0.0.1".to_string(),
        port: *endpoint,
        me: me.id.clone(),
        params: discovery::freenet::contract_params(&config.game),
        room_size: config.room_size,
        deploy_policy: config.deploy_policy,
        timing: config.timing,
    };
    tokio::spawn(discovery::freenet::poll_peers(
        freenet_config.clone(),
        peers_tx,
    ));
    tokio::spawn(async move {
        let client = discovery::freenet::connect_retry(&freenet_config, &target_rx).await;
        discovery::freenet::run_lobby(&freenet_config, client, target_rx, lobby_tx).await;
    });
    let mut tick = tokio::time::interval(timing.tick.max(Duration::from_secs(1)));
    loop {
        let input = tokio::select! {
            Some(command) = commands.recv() => Input::Command(command),
            Some(event) = net.events.recv() => Input::NetEvent(event),
            Some(lobby) = lobbies.recv() => Input::LobbyUpdated(lobby),
            Some(connected) = peers.recv() => Input::FreenetPeers(connected),
            Ok(()) = net.observed.changed() => {
                let Some(addrs) = net.observed.borrow_and_update().clone() else {
                    continue;
                };
                Input::OwnAddrsChanged(discovery::libp2p::dialable(addrs))
            }
            _ = tick.tick() => Input::Tick,
        };
        let outputs =
            discovery::state_machine::update(&mut state, input, discovery::UnixTime::now());
        flush(outputs, &net.commands, &events);
        let target = discovery::state_machine::heartbeat_target(&state);
        target_tx.send_if_modified(|current| {
            let changed = *current != target;
            if changed {
                current.clone_from(&target);
            }
            changed
        });
        let _ = snapshots.send(discovery::state_machine::snapshot(&state));
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
