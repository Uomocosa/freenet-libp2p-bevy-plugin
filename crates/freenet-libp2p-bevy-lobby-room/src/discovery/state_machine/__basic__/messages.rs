use bevy::prelude::Message;

use crate::discovery;
use crate::p2p;

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub enum Output {
    Notify(discovery::Event),
    Libp2pCommand(p2p::Libp2pCommand),
}
