use crate::discovery;

pub async fn refresh(
    lobby_contract: &mut discovery::freenet::LobbyContract,
) -> Result<discovery::Lobby, discovery::Error> {
    let instance_id = *lobby_contract.key.id();
    let fresh = discovery::freenet::fetch(&mut lobby_contract.client, instance_id).await?;
    discovery::freenet::merge_cached_lobby(lobby_contract, fresh);
    Ok(lobby_contract.lobby.clone())
}

// no test_usage necessary
