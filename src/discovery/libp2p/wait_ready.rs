use std::time::Duration;

use tokio::sync::watch::Receiver;

use crate::net_id;

const READY_TIMEOUT: Duration = Duration::from_secs(120);

pub async fn wait_ready(ready: &mut Receiver<Option<net_id::Peer>>) -> Option<net_id::Peer> {
    let wait = ready.wait_for(Option::is_some);
    match tokio::time::timeout(READY_TIMEOUT, wait).await {
        Ok(Ok(value)) => value.clone(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::wait_ready;
    use crate::net_id;

    #[tokio::test]
    async fn test_usage() {
        let ready = net_id::Peer {
            id: net_id::PeerId::from("me"),
            addrs: Vec::new(),
        };
        let (_tx, mut rx) = tokio::sync::watch::channel(Some(ready.clone()));
        assert_eq!(wait_ready(&mut rx).await, Some(ready));
    }
}
