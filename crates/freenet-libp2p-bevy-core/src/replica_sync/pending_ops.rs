use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

use crate::replica_sync;

#[derive(Resource, Deref, DerefMut)]
pub struct PendingOps<Replica: replica_sync::traits::ReplicatedState> {
    pub ops: Vec<Replica::Op>,
}

impl<Replica: replica_sync::traits::ReplicatedState> Default for PendingOps<Replica> {
    fn default() -> Self {
        Self { ops: Vec::new() }
    }
}

#[rustfmt::skip]
impl<Replica: replica_sync::traits::ReplicatedState> PendingOps<Replica> {
    pub fn send(&mut self, op: Replica::Op) { self.push(op); }
    pub fn take_all(&mut self) -> Vec<Replica::Op> { self.split_off(0) }
    pub fn restore(&mut self, ops: Vec<Replica::Op>) { self.extend(ops); }
}

// no test_usage necessary
