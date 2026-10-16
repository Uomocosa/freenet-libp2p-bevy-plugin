use tokio::sync::watch::Receiver;
use tracing::{debug, warn};

use crate::discovery;
use crate::net_id;
use discovery::freenet::basic::constants;

pub async fn connect_retry(
    host: &str,
    port: u16,
    params: &[u8],
    me: &net_id::PeerId,
    capacity: u16,
    target: &Receiver<Option<discovery::state_machine::PublishTarget>>,
    timing: discovery::Timing,
) -> discovery::freenet::LobbyClient {
    let wasm = discovery::freenet::contract_wasm();
    let mut client = connect_socket(host, port).await;
    let mut tries = 0_u32;
    loop {
        tries = tries.saturating_add(1);
        let current = target.borrow().clone();
        let deploy = if current.is_some() {
            discovery::freenet::DeployPolicy::FetchOrDeploy
        } else {
            discovery::freenet::DeployPolicy::FetchOnly
        };
        let board = initial_board(me, capacity, current.as_ref());
        let result =
            discovery::freenet::attach(&mut client, wasm, params, deploy, timing.ring, &board)
                .await;
        match result {
            Ok((key, lobby)) => return discovery::freenet::LobbyClient { client, key, lobby },
            Err(discovery::Error::Timeout(step)) => {
                warn!(target: "room_lobby", tries = tries, step, "lobby {step} timed out, reconnecting");
                client = connect_socket(host, port).await;
            }
            Err(discovery::Error::ContractNotFound) => {
                debug!(target: "room_lobby", tries = tries, "lobby board absent and not in a room, waiting");
                tokio::time::sleep(constants::MISSING_RETRY).await;
            }
            Err(e) => {
                warn!(target: "room_lobby", tries = tries, error = %e, "lobby board fetch failed, retrying");
                tokio::time::sleep(constants::CONNECT_RETRY).await;
            }
        }
    }
}

// needed helper: opens the local freenet websocket, retrying until it accepts
async fn connect_socket(host: &str, port: u16) -> discovery::freenet::Client {
    loop {
        match discovery::freenet::Client::connect(host, port).await {
            Ok(client) => return client,
            Err(e) => {
                warn!(target: "room_lobby", error = %e, "freenet websocket connect failed, retrying");
                tokio::time::sleep(constants::CONNECT_RETRY).await;
            }
        }
    }
}

// needed helper: seeds a fresh board with our own presence so readers never see it empty
fn initial_board(
    me: &net_id::PeerId,
    capacity: u16,
    target: Option<&discovery::state_machine::PublishTarget>,
) -> discovery::Lobby {
    let Some(target) = target else {
        return discovery::Lobby::new();
    };
    let presence = discovery::Presence {
        addrs: target.addrs.clone(),
        updated_at: discovery::UnixTime::now(),
    };
    let members = std::collections::BTreeMap::from([(me.clone(), presence)]);
    discovery::Lobby::from([(
        target.room.clone(),
        discovery::RoomEntry { capacity, members },
    )])
}

// no test_usage necessary
