use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::watch::Receiver;

use crate::net_id;
use crate::p2p;

pub struct Link {
    pub commands: UnboundedSender<p2p::NetCommand>,
    pub events: UnboundedReceiver<p2p::NetEvent>,
    pub ready: Receiver<Option<net_id::Peer>>,
    pub observed: Receiver<Option<Vec<net_id::PeerAddr>>>,
}
