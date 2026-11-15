use std::sync::Mutex;

use bevy::prelude::Resource;
use derive_more::Deref;
use tokio::sync::broadcast;

use atomic_delegate_macros::atomic_delegates;

use crate::p2p;

#[derive(Resource, Deref)]
pub struct Libp2pEvents {
    pub receiver: Mutex<broadcast::Receiver<p2p::Libp2pEvent>>,
}

#[atomic_delegates]
impl Libp2pEvents {
    pub fn drain(&self) -> Vec<p2p::Libp2pEvent> {}
}

// no test_usage necessary
