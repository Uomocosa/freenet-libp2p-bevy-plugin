use crate::discovery;

pub async fn refresh(
    lobby_client: &mut discovery::freenet::LobbyClient,
) -> Result<discovery::Lobby, discovery::Error> {
    let instance_id = *lobby_client.key.id();
    let fresh = discovery::freenet::fetch(&mut lobby_client.client, instance_id).await?;
    discovery::freenet::cache_lobby(lobby_client, fresh);
    Ok(lobby_client.lobby.clone())
}

// no test_usage necessary
