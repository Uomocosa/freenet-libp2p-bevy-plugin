use tokio::sync::mpsc::UnboundedSender;

use crate::discovery;
use crate::identity;
use discovery::freenet::basic::constants;

pub async fn poll_peers(
    freenet_config: discovery::freenet::Config,
    peers_tx: UnboundedSender<Vec<identity::FreenetPeer>>,
) {
    let mut client =
        discovery::freenet::connect_socket(&freenet_config.host, freenet_config.port).await;
    let mut interval = tokio::time::interval(discovery::TICK);
    loop {
        interval.tick().await;
        let reply = tokio::time::timeout(
            constants::REQUEST_TIMEOUT,
            discovery::freenet::connected_peers(&mut client),
        )
        .await;
        if let Ok(Ok(peers)) = reply
            && peers_tx.send(peers).is_err()
        {
            return;
        }
    }
}

// no test_usage necessary
