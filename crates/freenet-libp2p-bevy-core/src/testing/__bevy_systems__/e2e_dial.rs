use bevy::prelude::*;
use bevy::remote::{BrpError, BrpResult};
use serde_json::Value;

use crate::identity;
use crate::p2p;

pub fn e2e_dial(
    In(params): In<Option<Value>>,
    sender: Option<Res<p2p::Libp2pCommandSender>>,
) -> BrpResult {
    let addrs: String =
        serde_json::from_value(params.unwrap_or(Value::Null)).map_err(BrpError::internal)?;
    let mut parsed = Vec::new();
    for addr in addrs.split_whitespace() {
        let peer = identity::Peer::try_from(addr)
            .map_err(|error| BrpError::internal(format!("bad dial address {addr}: {error}")))?;
        parsed.push(peer);
    }
    let Some(first) = parsed.first() else {
        return Err(BrpError::internal("dial needs an address"));
    };
    let peer = identity::Peer {
        id: first.id.clone(),
        addrs: parsed.iter().flat_map(|peer| peer.addrs.clone()).collect(),
    };
    let Some(sender) = sender else {
        return Err(BrpError::internal("no libp2p command channel"));
    };
    sender
        .send(p2p::Libp2pCommand::Dial(peer))
        .map_err(|error| BrpError::internal(error.to_string()))?;
    Ok(Value::Null)
}
