use std::time::Duration;

use tokio::sync::mpsc::UnboundedSender;

use crate::discovery;
use crate::net_id;
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
    let me = net_id::Peer {
        addrs: discovery::libp2p::dialable(ready.addrs),
        id: ready.id,
    };
    let timing = config.timing;
    let capacity = config.capacity;
    let mut state = State::new(me.clone(), timing);
    let (target_tx, target_rx) = tokio::sync::watch::channel(None);
    let (lobby_tx, mut lobbies) = tokio::sync::mpsc::unbounded_channel();
    let params = discovery::freenet::contract_params(&config.game_name, &config.token);
    tokio::spawn(async move {
        let client =
            discovery::freenet::connect_retry("127.0.0.1", *endpoint, &params, &target_rx).await;
        discovery::freenet::run_lobby(client, me.id, capacity, timing, target_rx, lobby_tx).await;
    });
    let mut tick = tokio::time::interval(timing.tick.max(Duration::from_secs(1)));
    loop {
        let input = tokio::select! {
            Some(command) = commands.recv() => Input::Command(command),
            Some(event) = net.events.recv() => Input::NetEvent(event),
            Some(lobby) = lobbies.recv() => Input::LobbyUpdated(lobby),
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
        let target = discovery::state_machine::publish_target(&state);
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
