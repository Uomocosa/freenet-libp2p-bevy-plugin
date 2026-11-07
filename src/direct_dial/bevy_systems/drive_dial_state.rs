use bevy::prelude::*;

use crate::direct_dial;
use crate::discovery;
use crate::identity;
use crate::p2p;
use direct_dial::state_machine::{DialState, Input};

pub fn drive_dial_state(
    mut tap: Option<ResMut<direct_dial::DialEventTap>>,
    mut dial_state: Option<ResMut<DialState>>,
    mut book: Option<ResMut<direct_dial::PeerBook>>,
    config: Res<direct_dial::Config>,
    mut pending: Option<ResMut<p2p::PendingLibp2pCommands>>,
) {
    let config = config.into_inner();
    let Some(tap_rx) = tap.as_deref_mut() else {
        return;
    };
    let Some(dial) = dial_state.as_deref_mut() else {
        return;
    };
    let Some(book) = book.as_deref_mut() else {
        return;
    };
    let Some(pending) = pending.as_deref_mut() else {
        return;
    };
    let now = discovery::UnixTime::now();
    let mut inputs: Vec<Input> = Vec::new();
    while let Ok(event) = tap_rx.try_recv() {
        match event {
            p2p::Libp2pEvent::PeerConnected(id) => {
                inputs.push(Input::PeerConnected(id));
            }
            p2p::Libp2pEvent::PeerDisconnected(id) => {
                inputs.push(Input::PeerDisconnected(id));
            }
            p2p::Libp2pEvent::DialFailed { peer_id, .. } => {
                if let Some(entry) = dial.entries.iter().find(|entry| entry.peer.id == peer_id) {
                    inputs.push(Input::DialFailed(entry.peer.clone()));
                } else {
                    tracing::debug!(target: "direct_dial", peer = %peer_id.as_str(), "dial failed for unknown peer");
                }
            }
            p2p::Libp2pEvent::PeerListenAddrs { peer_id, addrs } => {
                inputs.push(Input::PeerListenAddrs(identity::Peer {
                    id: peer_id,
                    addrs,
                }));
            }
            _ => {}
        }
    }
    for entry in book.iter() {
        if entry.kind != direct_dial::KnownPeerKind::Friend {
            continue;
        }
        let known = dial
            .entries
            .iter()
            .find(|known| known.peer.id == entry.peer.id);
        let needs_add = known.is_none_or(|known| {
            known.kind != direct_dial::KnownPeerKind::Friend || known.peer.addrs != entry.peer.addrs
        });
        if needs_add {
            inputs.push(Input::FriendAdded(entry.peer.clone()));
        }
    }
    for entry in &dial.entries {
        if entry.kind != direct_dial::KnownPeerKind::Friend {
            continue;
        }
        let still_friend = book.iter().any(|known| known.peer.id == entry.peer.id);
        if !still_friend {
            inputs.push(Input::FriendRemoved(entry.peer.id.clone()));
        }
    }
    inputs.push(Input::Tick);
    let mut outputs: Vec<direct_dial::state_machine::Output> = Vec::new();
    for input in inputs {
        outputs.extend(dial.update(input, now));
    }
    for output in outputs {
        match output {
            direct_dial::state_machine::Output::Dial(peer) => {
                pending.push(p2p::Libp2pCommand::Dial(peer));
            }
            direct_dial::state_machine::Output::SavePeerBook(next) => {
                book.clone_from(&next);
                if let Some(path) = config.as_ref()
                    && let Err(error) = book.save_to_file(path)
                {
                    tracing::error!(target: "direct_dial", "peer book save failed: {error}");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::drive_dial_state;
    use crate::direct_dial;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;

    fn peer_with(name: &str, addr: &str) -> identity::Peer {
        identity::Peer {
            id: identity::PeerId::from(name),
            addrs: vec![identity::PeerAddr::from(addr)],
        }
    }

    fn app_with_book(
        book: &direct_dial::PeerBook,
    ) -> (App, tokio::sync::broadcast::Sender<p2p::Libp2pEvent>) {
        let channel = p2p::Libp2pEventChannel::default();
        let tx = channel.tx.clone();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(channel);
        let now = discovery::UnixTime::now();
        app.insert_resource(book.clone());
        app.insert_resource(direct_dial::state_machine::DialState::from_peer_book(
            book, now,
        ));
        let rx = app
            .world()
            .resource::<p2p::Libp2pEventChannel>()
            .subscribe();
        app.insert_resource(direct_dial::DialEventTap(rx));
        app.insert_resource(direct_dial::Config::from(None));
        app.insert_resource(p2p::PendingLibp2pCommands::default());
        app.add_systems(Update, drive_dial_state);
        (app, tx)
    }

    #[test]
    fn test_usage() {
        let mut book = direct_dial::PeerBook::default();
        assert!(book.add_friend(peer_with("alice", "/ip4/127.0.0.1/tcp/4001")));
        let (mut app, _tx) = app_with_book(&book);
        app.update();
        assert!(
            !app.world()
                .resource::<p2p::PendingLibp2pCommands>()
                .is_empty()
        );
    }

    #[test]
    fn failure_maps_to_the_right_peer() {
        let mut book = direct_dial::PeerBook::default();
        assert!(book.add_friend(peer_with("alice", "/ip4/127.0.0.1/tcp/4001")));
        assert!(book.add_friend(peer_with("bob", "/ip4/127.0.0.1/tcp/4002")));
        let (mut app, tx) = app_with_book(&book);
        app.update();
        tx.send(p2p::Libp2pEvent::DialFailed {
            peer_id: identity::PeerId::from("alice"),
            reason: String::from("refused"),
        })
        .expect("send");
        app.update();
        let dial = app
            .world()
            .resource::<direct_dial::state_machine::DialState>();
        let alice_attempts = dial
            .entries
            .iter()
            .find(|entry| entry.peer.id == identity::PeerId::from("alice"))
            .map(|entry| entry.failed_attempts)
            .unwrap_or_default();
        let bob_attempts = dial
            .entries
            .iter()
            .find(|entry| entry.peer.id == identity::PeerId::from("bob"))
            .map(|entry| entry.failed_attempts)
            .unwrap_or_default();
        assert_eq!(alice_attempts, 1);
        assert_eq!(bob_attempts, 0);
    }

    #[test]
    fn inbound_ephemeral_address_is_not_saved() {
        let book = direct_dial::PeerBook::default();
        let (mut app, tx) = app_with_book(&book);
        let mallory = identity::PeerId::from("mallory");
        tx.send(p2p::Libp2pEvent::Libp2pConnectionEstablished {
            peer_id: mallory.clone(),
            kind: p2p::ConnectionKind::Tcp,
        })
        .expect("send");
        tx.send(p2p::Libp2pEvent::PeerConnected(mallory.clone()))
            .expect("send");
        app.update();
        assert!(app.world().resource::<direct_dial::PeerBook>().is_empty());
        let listen = vec![identity::PeerAddr::from("/ip4/10.0.0.5/tcp/4001")];
        tx.send(p2p::Libp2pEvent::PeerListenAddrs {
            peer_id: mallory.clone(),
            addrs: listen.clone(),
        })
        .expect("send");
        app.update();
        let book = app.world().resource::<direct_dial::PeerBook>();
        assert_eq!(book.len(), 1);
        assert_eq!(book[0].peer.id, mallory);
        assert_eq!(book[0].peer.addrs, listen);
    }
}
