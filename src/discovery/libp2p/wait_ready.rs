use std::time::Duration;

use tokio::sync::watch::Receiver;

use crate::identity;

const READY_TIMEOUT: Duration = Duration::from_secs(120);

pub async fn wait_ready(ready: &mut Receiver<Option<identity::Peer>>) -> Option<identity::Peer> {
    let wait = ready.wait_for(Option::is_some);
    match tokio::time::timeout(READY_TIMEOUT, wait).await {
        Ok(Ok(value)) => value.clone(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::wait_ready;
    use crate::identity;

    #[tokio::test]
    async fn test_usage() {
        let ready = identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        };
        let (_tx, mut rx) = tokio::sync::watch::channel(Some(ready.clone()));
        assert_eq!(wait_ready(&mut rx).await, Some(ready));
    }
}
