use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

use crate::p2p;

#[derive(Resource, Debug, Deref, DerefMut)]
pub struct PeerEventTap(pub tokio::sync::broadcast::Receiver<p2p::Libp2pEvent>);
