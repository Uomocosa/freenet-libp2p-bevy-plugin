use tokio::sync::watch::Receiver;
use tracing::{debug, warn};

use crate::discovery;
use crate::identity;
use discovery::freenet::basic::constants;

pub async fn connect_retry(
    freenet_config: &discovery::freenet::Config,
    target: &Receiver<Vec<discovery::state_machine::HeartbeatTarget>>,
) -> discovery::freenet::LobbyContractSubscription {
    let wasm = discovery::freenet::contract_wasm();
    let mut client =
        discovery::freenet::connect_socket(&freenet_config.host, freenet_config.port).await;
    let mut tries = 0_u32;
    loop {
        tries = tries.saturating_add(1);
        let current = target.borrow().clone();
        let seed = initial_lobby(
            &freenet_config.me,
            freenet_config.discovery.room_size,
            current.first(),
        );
        let result = discovery::freenet::attach(
            &mut client,
            wasm,
            &freenet_config.params,
            freenet_config.discovery.deploy_policy,
            constants::RING_WAIT,
            &seed,
        )
        .await;
        match result {
            Ok((key, lobby)) => {
                return discovery::freenet::LobbyContractSubscription { client, key, lobby };
            }
            Err(discovery::Error::Timeout(step)) => {
                warn!(target: "discovery", tries = tries, step = %step, "lobby {step} timed out, reconnecting");
                client =
                    discovery::freenet::connect_socket(&freenet_config.host, freenet_config.port)
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
        presence: target.presence,
    };
    let members = std::collections::BTreeMap::from([(me.clone(), room_member)]);
    discovery::Lobby::from([(
        target.room.clone(),
        discovery::Room {
            room_size,
            created_at: target.created_at,
            members,
        },
    )])
}

// no test_usage necessary
