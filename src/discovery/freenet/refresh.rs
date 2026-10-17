use crate::discovery;

pub async fn refresh(
    subscription: &mut discovery::freenet::LobbyContractSubscription,
) -> Result<discovery::Lobby, discovery::Error> {
    let instance_id = *subscription.key.id();
    let fresh = discovery::freenet::fetch(&mut subscription.client, instance_id).await?;
    subscription.lobby.merge(fresh);
    Ok(subscription.lobby.clone())
}

// no test_usage necessary
