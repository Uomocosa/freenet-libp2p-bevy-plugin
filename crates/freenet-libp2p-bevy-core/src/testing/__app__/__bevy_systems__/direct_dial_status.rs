use bevy::prelude::*;
use bevy::remote::{BrpError, BrpResult};
use serde_json::Value;

use crate::direct_dial;
use crate::identity;
use crate::p2p;
use crate::peers;
use crate::testing;

pub fn direct_dial_status(
    In(_params): In<Option<Value>>,
    ready: Option<Res<p2p::ReadyChannel>>,
    dial: Option<Res<direct_dial::state_machine::DialState>>,
    connected: Query<&peers::Peer, With<peers::Connected>>,
) -> BrpResult {
    let own: Option<identity::Peer> = ready.and_then(|ready| ready.ready_rx.borrow().clone());
    let peer_id = own
        .as_ref()
        .map(|peer| peer.id.to_string())
        .unwrap_or_default();
    let listen_addrs: Vec<String> = own
        .as_ref()
        .map(|peer| {
            identity::PeerAddr::dialable(peer.addrs.clone())
                .iter()
                .map(|addr| format!("{addr}/p2p/{}", peer.id))
                .collect()
        })
        .unwrap_or_default();
    let connected = connected.iter().map(|peer| (**peer).to_string()).collect();
    let book = dial
        .map(|dial| {
            dial.entries
                .iter()
                .map(|entry| testing::DirectDialBookEntry {
                    peer_id: entry.peer.id.to_string(),
                    kind: entry.kind,
                })
                .collect()
        })
        .unwrap_or_default();
    let status = testing::DirectDialStatus {
        peer_id,
        listen_addrs,
        connected,
        book,
    };
    serde_json::to_value(status).map_err(BrpError::internal)
}
