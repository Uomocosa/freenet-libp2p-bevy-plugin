use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::watch::Receiver;

use crate::identity;
use crate::p2p;

pub struct P2PChannels {
    pub commands: UnboundedSender<p2p::NetCommand>,
    pub events: UnboundedReceiver<p2p::NetEvent>,
    pub ready: Receiver<Option<identity::Peer>>,
    pub observed: Receiver<Option<Vec<identity::PeerAddr>>>,
}
