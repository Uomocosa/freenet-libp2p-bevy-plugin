use std::net::SocketAddr;

use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};

use crate::freenet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub struct FreenetPeer {
    pub id: freenet::FreenetPeerId,
    pub addr: SocketAddr,
}
