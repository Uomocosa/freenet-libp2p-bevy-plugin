use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::watch::Receiver as WatchReceiver;

use crate::identity;
use crate::p2p;

pub struct SwarmChannels {
    pub commands: UnboundedSender<p2p::Libp2pCommand>,
    pub events: Receiver<p2p::Libp2pEvent>,
    pub ready: WatchReceiver<Option<identity::Peer>>,
    pub observed: WatchReceiver<Option<Vec<identity::PeerAddr>>>,
}
