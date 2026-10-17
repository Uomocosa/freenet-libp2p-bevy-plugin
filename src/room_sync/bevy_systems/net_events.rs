use std::sync::Mutex;

use bevy::prelude::Resource;
use derive_more::Deref;
use tokio::sync::broadcast;

use crate::p2p;

#[derive(Resource, Deref)]
pub struct NetEvents {
    pub receiver: Mutex<broadcast::Receiver<p2p::NetEvent>>,
}

#[rustfmt::skip]
impl NetEvents {
    pub fn drain(&self) -> Vec<p2p::NetEvent> {
        let Ok(mut receiver) = self.receiver.lock() else { return Vec::new(); };
        std::iter::from_fn(|| receiver.try_recv().ok()).collect()
    }
}

// no test_usage necessary
