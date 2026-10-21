use std::collections::BTreeMap;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest};
use freenet_stdlib::prelude::{StateDelta, UpdateData};

use crate::discovery;
use crate::identity;

pub fn publish_heartbeat(
    subscription: &mut discovery::freenet::LobbyContractSubscription,
    room: &identity::RoomName,
    me: &identity::PeerId,
    room_member: discovery::RoomMember,
    room_size: discovery::RoomSize,
    created_at: discovery::UnixTime,
) -> Result<(), discovery::Error> {
    let members = BTreeMap::from([(me.clone(), room_member)]);
    let update = discovery::Lobby::from([(
        room.clone(),
        discovery::Room {
            room_size,
            created_at,
            members,
        },
    )]);
    let request = ContractRequest::Update {
        key: subscription.key,
        data: UpdateData::Delta(StateDelta::from(bincode::serialize(&update)?)),
    };
    subscription
        .client
        .send(&ClientRequest::ContractOp(request))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use freenet_stdlib::prelude::{ContractCode, ContractKey, Parameters, WrappedContract};

    use super::publish_heartbeat;
    use crate::{discovery, identity};

    fn contract_key() -> ContractKey {
        WrappedContract::new(
            Arc::new(ContractCode::from(
                discovery::freenet::contract_wasm().to_vec(),
            )),
            Parameters::from(Vec::new()),
        )
        .key
    }

    #[test]
    fn test_usage() {
        let (write, mut write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (_read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        let mut subscription = discovery::freenet::LobbyContractSubscription {
            client: discovery::freenet::Client { write, read },
            key: contract_key(),
            lobby: discovery::Lobby::default(),
        };
        let room_member = discovery::RoomMember {
            peer: identity::Peer {
                id: identity::PeerId::from("me"),
                addrs: Vec::new(),
            },
            updated_at: discovery::UnixTime::from_secs(1),
            display_name: None,
            presence: discovery::Presence::InRoom,
        };
        assert!(
            publish_heartbeat(
                &mut subscription,
                &identity::RoomName::from("room-a"),
                &identity::PeerId::from("me"),
                room_member,
                discovery::RoomSize::DEFAULT,
                discovery::UnixTime::from_secs(0),
            )
            .is_ok()
        );
        assert!(write_rx.try_recv().is_ok());
    }
}
