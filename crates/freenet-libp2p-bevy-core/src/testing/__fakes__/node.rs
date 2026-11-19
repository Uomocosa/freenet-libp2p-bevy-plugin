use crate::p2p;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub struct Node<Payload: p2p::traits::Message> {
    pub commands: UnboundedSender<p2p::Command<Payload>>,
    pub libp2p_commands: UnboundedSender<p2p::Libp2pCommand>,
    pub events: UnboundedReceiver<p2p::Event<Payload>>,
    pub handle: tokio::task::JoinHandle<()>,
}
