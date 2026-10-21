use crate::discovery;

pub async fn refresh(
    subscription: &mut discovery::freenet::LobbyContractSubscription,
) -> Result<discovery::Lobby, discovery::Error> {
    let instance_id = *subscription.key.id();
    let fresh =
        discovery::freenet::fetch_and_subscribe(&mut subscription.client, instance_id).await?;
    subscription.lobby.merge(fresh);
    Ok(subscription.lobby.clone())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use freenet_stdlib::client_api::{ContractResponse, HostResponse};
    use freenet_stdlib::prelude::{
        ContractCode, ContractKey, Parameters, WrappedContract, WrappedState,
    };

    use super::refresh;
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

    #[tokio::test]
    async fn test_usage() {
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        let key = contract_key();
        let fresh = discovery::Lobby::from([(
            identity::RoomName::from("room-a"),
            discovery::Room {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: discovery::UnixTime::from_secs(0),
                members: std::collections::BTreeMap::from([(
                    identity::PeerId::from("peer"),
                    discovery::RoomMember {
                        peer: identity::Peer {
                            id: identity::PeerId::from("peer"),
                            addrs: Vec::new(),
                        },
                        updated_at: discovery::UnixTime::from_secs(1),
                        display_name: None,
                        presence: discovery::Presence::InRoom,
                    },
                )]),
            },
        )]);
        read_tx
            .send(Ok(HostResponse::ContractResponse(
                ContractResponse::GetResponse {
                    key,
                    contract: None,
                    state: WrappedState::new(bincode::serialize(&fresh).unwrap_or_default()),
                },
            )))
            .ok();
        let mut subscription = discovery::freenet::LobbyContractSubscription {
            client: discovery::freenet::Client { write, read },
            key,
            lobby: discovery::Lobby::default(),
        };
        assert_eq!(refresh(&mut subscription).await.ok(), Some(fresh.clone()));
        assert_eq!(subscription.lobby, fresh);
    }
}
