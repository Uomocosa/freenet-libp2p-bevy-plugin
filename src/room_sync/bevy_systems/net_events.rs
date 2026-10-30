use std::sync::Mutex;

use bevy::prelude::Resource;
use derive_more::Deref;
use tokio::sync::broadcast;

use atomic_delegate_macros::atomic_delegates;

use crate::p2p;

#[derive(Resource, Deref)]
pub struct NetEvents {
    pub receiver: Mutex<broadcast::Receiver<p2p::NetEvent>>,
}

#[atomic_delegates]
impl NetEvents {
    pub fn drain(&self) -> Vec<p2p::NetEvent> {}
}

// no test_usage necessary
