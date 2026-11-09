use std::net::SocketAddr;

use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};

use crate::identity;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub struct FreenetPeer {
    pub id: identity::FreenetPeerId,
    pub addr: SocketAddr,
}
