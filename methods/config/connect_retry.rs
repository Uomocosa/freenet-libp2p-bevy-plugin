use tokio::sync::watch::Receiver;
use tracing::{debug, warn};

use crate::discovery;
use crate::identity;
use discovery::freenet::basic::constants;

pub async fn connect_retry(
    state: &discovery::freenet::Config,
    target: &Receiver<Vec<discovery::state_machine::HeartbeatTarget>>,
) -> discovery::freenet::LobbyContractSubscription {
    let wasm = discovery::freenet::LOBBY_CONTRACT_WASM;
    let mut client =
        discovery::freenet::Client::from_host_and_port_with_retry(&state.host, state.port).await;
    let mut tries = 0_u32;
    loop {
        tries = tries.saturating_add(1);
        let current = target.borrow().clone();
        let seed = initial_lobby(&state.me, state.discovery.room_size, current.first());
        let result = client
            .attach(
                wasm,
                &state.params,
                state.discovery.deploy_policy,
                constants::RING_WAIT,
                &seed,
            )
            .await;
        match result {
            Ok((key, lobby)) => {
                return discovery::freenet::LobbyContractSubscription { client, key, lobby };
            }
            Err(discovery::Error::Timeout(kind)) => {
                warn!(target: "discovery", tries = tries, kind = %kind, "{kind} timed out, reconnecting");
                client = discovery::freenet::Client::from_host_and_port_with_retry(
                    &state.host,
                    state.port,
                )
                .await;
            }
            Err(discovery::Error::ContractNotFound) => {
                debug!(target: "discovery", tries = tries, "lobby absent and not in a room, waiting");
                tokio::time::sleep(constants::MISSING_RETRY).await;
            }
            Err(e) => {
                warn!(target: "discovery", tries = tries, error = %e, "lobby attach failed, retrying");
                tokio::time::sleep(constants::CONNECT_RETRY).await;
            }
        }
    }
}

// needed helper: seeds a fresh lobby with our own heartbeat so readers never see it empty
fn initial_lobby(
    me: &identity::PeerId,
    room_size: discovery::RoomSize,
    target: Option<&discovery::state_machine::HeartbeatTarget>,
) -> discovery::Lobby {
    let Some(target) = target else {
        return discovery::Lobby::default();
    };
    let room_member = discovery::RoomMember {
        peer: identity::Peer {
            id: me.clone(),
            addrs: target.addrs.clone(),
        },
        updated_at: discovery::UnixTime::now(),
        display_name: Some(target.display_name.clone()),
        membership: target.membership,
    };
    let members = std::collections::BTreeMap::from([(me.clone(), room_member)]);
    discovery::Lobby::from([(
        target.room.clone(),
        discovery::LobbyRoom {
            room_size,
            created_at: target.created_at,
            members,
        },
    )])
}

#[cfg(test)]
mod tests {
    use super::connect_retry;

    use crate::discovery;
    use crate::identity;

    #[tokio::test]
    async fn test_usage() {
        let config = discovery::freenet::Config {
            host: "127.0.0.1".to_string(),
            port: 0,
            me: identity::PeerId::from("me"),
            params: Vec::new(),
            discovery: discovery::Config::default(),
        };
        let (_target_tx, target_rx) = tokio::sync::watch::channel(Vec::new());
        let handle = tokio::spawn(async move { connect_retry(&config, &target_rx).await });
        handle.abort();
    }
}
